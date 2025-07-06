use murmurhash3::murmurhash3_x86_32;
use html2text::from_read;
use std::io::Write;
use std::io::Read;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;
use std::fs::OpenOptions;

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

fn parse(url: &str) -> (Vec<String>, String) {
    let mut links = Vec::new();
    let mut result = url.split("/");
    let protocol = result.next().unwrap();
    result.next().unwrap();
    let domain = result.next().unwrap();

    let html = reqwest::blocking::get(url).unwrap().text().unwrap();

    let html_text = from_read(html.as_bytes(), 200).unwrap();
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
   
    (links, html_text)
}

fn open_awled_file() -> File {
    return OpenOptions::new().read(true).write(true).create(true).append(true).open("awled_html.bin").unwrap();
}

/**
 * 将爬到的网页存起来
 * id、size 为u32
 * 每一个网页存储的格式是 id\tsize\thtml\r\n\r\n
 */
fn save_awled_html(id: u32, html: &str) {
    let file = open_awled_file();
    let mut write = BufWriter::new(file);
    write.write(&id.to_le_bytes()).unwrap();
    write.write("\t".as_bytes()).unwrap();
    write.write(&(html.len() as u32).to_le_bytes()).unwrap();
    write.write("\t".as_bytes()).unwrap();
    write.write(&html.as_bytes()).unwrap();
    write.write("\r\n\r\n".as_bytes()).unwrap();
    write.flush().unwrap(); // 确保数据被写入文件
}

/**
 * 读取爬过的网页
 */
fn read_awled_html() {
    let file = open_awled_file();
    let mut read = BufReader::new(file);
    let mut data = vec![];
    read.read_to_end(&mut data).unwrap();

    let mut pos = 0;
    while pos < data.len() {
        let id = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 5;
        let size = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 5;
        let html = String::from_utf8(data[pos..pos + size as usize].to_vec()).unwrap();
        pos += 4 + size as usize;
        println!("id: {}, size: {}, html: {}",  id, size, html);
    }
}

fn main() {
    let mut links = vec![];
    let mut bloom_filter = BloomFilter::new();
    let mut html_id: u32 = 0;
    
    let mut wait_awled: Vec<String> = vec!["https://chaihuibin.vercel.app".to_string()];
    bloom_filter.add("https://chaihuibin.vercel.app");
    'outer: loop {
        let mut new_wait_links = vec![];
        for i in 0..wait_awled.len() {
            let (awled_links, html_text) = parse(&wait_awled[i]);
            for j in 0..awled_links.len() {

                // 超过10个就不再爬了
                if links.len() + wait_awled.len() + new_wait_links.len() > 10 {
                    wait_awled.iter().for_each(|link| {links.push(link.clone())});
                    new_wait_links.iter().for_each(|link: &String| {links.push(link.clone())});
                    break 'outer;
                }

                if bloom_filter.add(&awled_links[j]) {
                    new_wait_links.push(awled_links[j].to_string());
                    save_awled_html(html_id, &html_text);
                    html_id += 1;
                }
            }
        }
        wait_awled.iter().for_each(|link| {links.push(link.clone())});
        wait_awled = new_wait_links;
    }
    
    for i in 0..links.len() {
        println!("{}: {}", i, links[i])
    }
}