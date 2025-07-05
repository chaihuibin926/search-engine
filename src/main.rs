use murmurhash3::murmurhash3_x86_32;

struct BitMap {
    data: Vec<u8>,
}

impl BitMap {
    fn new(size: usize) -> Self {
        Self { data: vec![0; size / 8 + 1] }
    }

    fn set(&mut self, index: usize) {
        let data_index = index / 8;
        let bit_index = index % 8;
        self.data[data_index] |= 1 << bit_index;
    }

    fn get(&self, index: usize) -> bool {
        let data_index = index / 8;
        let bit_index = index % 8;
        return self.data[data_index] & (1 << bit_index) != 0;
    }
}

struct BloomFilter {
    bit_map: BitMap,
    sends: [u32; 2],
}

impl BloomFilter {
    fn new() -> Self {

        Self { bit_map: BitMap::new(u32::MAX as usize), sends: [0x9747B28C, 0x0747B28D] }
    }

    fn add(&mut self, data: &str) -> bool {
        let hash1 = murmurhash3_x86_32(data.as_bytes(), self.sends[0]);
        let hash2 = murmurhash3_x86_32(data.as_bytes(), self.sends[1]);

        if self.bit_map.get(hash1 as usize) && self.bit_map.get(hash2 as usize) {
            return false;
        } else {
            self.bit_map.set(hash1 as usize);
            self.bit_map.set(hash2 as usize);
            return true;
        }
    }
}

fn get_href_by_a_tag(a_html: &str) -> String {
    let href_start = a_html.find("href=\"");
    match href_start {
        Some(href_start) => {
            let actual_start = href_start + 6;
            let href_end = a_html[actual_start..].find('"').expect("invaild href");
            a_html[actual_start..(actual_start + href_end)].to_string()
        },
        None => String::from(""),
    }
}

fn valid_href(href: &str) -> bool {
    if href.len() == 0 || href.eq("/") {
        return false 
    }
    if href.starts_with("http") || href.starts_with("/") {
        return true
    }
    false
}

fn get_html_by_address(url: &str) -> Vec<String> {
    println!("get_html_by_address: {}", url);
    let mut links = Vec::new();
    let mut result = url.split("/");
    let protocol = result.next().unwrap();
    result.next().unwrap();
    let domain = result.next().unwrap();

    let html = reqwest::blocking::get(url).unwrap().text().unwrap();
    println!("读取到html");
    // 在html中找到所有的a标签 获取href属性
    let mut pos = 0;
    while pos < html.len() {
        if let Some(a_tag_start) = html[pos..].find("<a ") {
            let actual_start = pos + a_tag_start;
            let a_tag_end = html[actual_start..].find(">").expect("invaild a tag");
            let actual_end = a_tag_end + actual_start + 1;
            let href = get_href_by_a_tag(&html[actual_start..actual_end]);
            if valid_href(&href) {
                if href.starts_with("/") {
                    links.push(format!("{protocol}//{domain}{href}"));
                } else {
                    links.push(href);
                }
            }
            pos = actual_end
        } else {
            break;
        }
    }
   
    links
}

fn main() {
    let mut links = vec![];
    let mut bloom_filter = BloomFilter::new();
    
    let mut temp: Vec<String> = vec!["https://chaihuibin.vercel.app".to_string()];
    'outer: loop {
        let mut new_links = vec![];
        for i in 0..temp.len() {
            let temp1 = get_html_by_address(&temp[i]);
            for j in 0..temp1.len() {

                // 超过50个就不再爬了
                if links.len() + temp.len() + new_links.len() > 50 {
                    temp.iter().for_each(|link| {links.push(link.clone())});
                    new_links.iter().for_each(|link: &String| {links.push(link.clone())});
                    break 'outer;
                }

                if bloom_filter.add(&temp1[j]) {
                    new_links.push(temp1[j].to_string());
                }
            }
        }
        temp.iter().for_each(|link| {links.push(link.clone())});
        temp = new_links;
    }
    temp.iter().for_each(|link| {links.push(link.clone())});
    
    for i in 0..links.len() {
        println!("{}: {}", i, links[i])
    }
}