use html2text::render::TrivialDecorator;
use murmurhash3::murmurhash3_x86_32;
use html2text::from_read;
use html2text::from_read_with_decorator;
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

/**
 * 校验href是否是一个可以跳转的href
 */
fn valid_href(href: &str) -> bool {
    if href.len() == 0 || href.eq("/") {
        return false 
    }
    if href.starts_with("http") || href.starts_with("/") {
        return true
    }
    false
}

/**
 * 解析目标网页，解析出网页内的links和网页的用户可见的文本内容
 */
fn parse(url: &str) -> (Vec<String>, String) {
    let mut links = Vec::new();
    let mut result = url.split("/");
    let protocol = result.next().unwrap();
    result.next().unwrap();
    let domain = result.next().unwrap();

    println!("开始请求地址{}", url);

    let html = match reqwest::blocking::get(url).unwrap().text() {
        Ok(html) => html,
        Err(e) => {
            println!("请求地址失败，目标地址{}，错误信息{}", url, e);
            return (vec![], String::from(""))
        }
    };

    println!("进入解析阶段");

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
   
    (links, html)
}

fn open_file(path: &str) -> File {
    return OpenOptions::new().read(true).write(true).create(true).append(true).open(path).unwrap();
}

/**
 * 将爬到的网页存起来
 * id、size 为u32
 * 每一个网页存储的格式是 id\tsize\thtml\r\n\r\n
 */
fn save_awled_html(id: u32, html: String, url: String) {
    let doc_raw_file = open_file("doc_raw.bin");
    let doc_id_file = open_file("doc_id.bin");
    // 写入doc_id.bin
    let mut doc_id_write = BufWriter::new(doc_id_file);
    doc_id_write.write(&id.to_le_bytes()).unwrap();
    doc_id_write.write("\t".as_bytes()).unwrap();
    doc_id_write.write(url.as_bytes()).unwrap();
    doc_id_write.write("\r\n".as_bytes()).unwrap();
    // 写入doc_raw.bin
    let mut write = BufWriter::new(doc_raw_file);
    write.write(&id.to_le_bytes()).unwrap();
    write.write("\t".as_bytes()).unwrap();
    write.write(&(html.len() as u32).to_le_bytes()).unwrap();
    write.write("\t".as_bytes()).unwrap();
    write.write(html.as_bytes()).unwrap();
    write.write("\r\n\r\n".as_bytes()).unwrap();
    write.flush().unwrap(); // 确保数据被写入文件
}

/**
 * 读取爬过的网页
 */
fn read_awled_html() -> Vec<(u32, u32, String)> {
    let file = open_file("doc_raw.bin");
    let mut read = BufReader::new(file);
    let mut data = vec![];
    read.read_to_end(&mut data).unwrap();

    let mut result = vec![];

    let mut pos = 0;
    while pos < data.len() {
        let id = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 5;
        let size = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 5;
        let html = String::from_utf8(data[pos..pos + size as usize].to_vec()).unwrap();

        // let html_text = from_read_with_decorator(
        //     html.as_bytes(),
        //     200,
        //     TrivialDecorator::new()
        // )
        // .unwrap();
        let html_text = from_read(
            html.as_bytes(),
            200,
        )
        .unwrap();

        pos += 4 + size as usize;
        result.push((id, size, html_text));
    }
    result
}


/**
 * 读取doc_id
 */
fn read_doc_id() -> Vec<(u32, String)> {
    let file = open_file("doc_id.bin");
    let mut read = BufReader::new(file);
    let mut data = vec![];
    
    read.read_to_end(&mut data).unwrap();

    let mut result = vec![];
    let mut pos = 0;
    
    while pos < data.len() {
        // 读取u32的id
        if pos + 4 > data.len() {
            break;
        }
        let id = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;
        
        // 跳过制表符
        if pos < data.len() && data[pos] == b'\t' {
            pos += 1;
        }
        
        // 读取URL字符串直到\r\n
        let url_start = pos;
        while pos < data.len() {
            if pos + 1 < data.len() && data[pos] == b'\r' && data[pos + 1] == b'\n' {
                break;
            }
            pos += 1;
        }
        
        if url_start < pos {
            let url = String::from_utf8(data[url_start..pos].to_vec()).unwrap();
            result.push((id, url));
        }
        
        // 跳过\r\n
        if pos + 1 < data.len() {
            pos += 2;
        }
    }
    
    result
}

fn run_awled() {
    let mut links = vec![];
    let mut bloom_filter = BloomFilter::new();
    let mut html_id: u32 = 0;
    
    let mut wait_awled: Vec<String> = vec!["https://www.woshipm.com/class/6238617.html".to_string()];
    bloom_filter.add("https://www.woshipm.com/class/6238617.html");
    loop {
        let mut new_wait_links = vec![];
        for i in 0..wait_awled.len() {
            let (awled_links, html_text) = parse(&wait_awled[i]);
            save_awled_html(html_id, html_text, wait_awled[i].clone());
            html_id += 1;
            for j in 0..awled_links.len() {
                if bloom_filter.add(&awled_links[j]) {
                    new_wait_links.push(awled_links[j].to_string());
                }
            }
        }
        wait_awled.iter().for_each(|link| {links.push(link.clone())});
        wait_awled = new_wait_links;
        if links.len() > 6 {
            break
        }
    }
    
    for i in 0..links.len() {
        println!("{}: {}", i, links[i])
    }
}

fn main() {
    run_awled();
    // let data = read_awled_html();
    // for i in 0..10 {
    //     println!("{}: {}", i, data[i].2)
    // }
}