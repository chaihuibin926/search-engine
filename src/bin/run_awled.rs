use std::io::{BufWriter, Write};
use mini_search_engine::open_file;
use mini_search_engine::BloomFilter;


fn main() {
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
        if links.len() > 5 {
            break
        }
    }
    
    for i in 0..links.len() {
        println!("{}: {}", i, links[i])
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
 * 将爬到的网页存起来
 * id、size 为u32
 * 每一个网页存储的格式是 id\tsize\thtml\r\n\r\n
 */
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

    let html: String = match reqwest::blocking::get(url).unwrap().text() {
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

