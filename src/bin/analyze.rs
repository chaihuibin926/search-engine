use html2text::from_read;
use mini_search_engine::open_file;
use std::{fs::{read_dir, read}, io::{BufReader, Read}};

fn main() {
    // let words = get_wrods();
    // let data = read_awled_html();
    // let html = data[0].2.clone();
    // let matched_words = match_words(&words, html);
    // println!("{:?}", matched_words);
}

// 匹配网页中的分词
fn match_words(words: &Vec<String>, html: String) -> Vec<String> {
    let mut result = vec![];
    let chars = html.chars().collect::<Vec<char>>();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        // println!("{}", i);
        let mut find = false;
        for j in i..len {
            let word = chars[i..i+len-j].iter().collect::<String>();
            if words.contains(&word) {
                i += word.chars().count();
                result.push(word);
                find = true;
                break;
            }
        }
        if !find {
            i += 1;
        }
    }

    result
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

// 获取分词列表
fn get_wrods() -> Vec<String> {
    let mut words = vec![];
    // 读取分词目录文件
    let dir = read_dir("data\\dictionaries").unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        let path: std::path::PathBuf = entry.path();
        let data = read(&path).unwrap();
        let data = String::from_utf8(data).unwrap();

        data.split("\r\n")
            .filter_map(|word| word.split("\t").next())
            .for_each(|word| words.push(word.to_string()));
    }
    words
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

mod test {
    use super::*;

    #[test]
    fn test_match_words() {
        let words1 = vec!["你好啊".to_string(), "xxx".to_string(), "我是真的xxx".to_string(), "真的爱上你".to_string()];
        let html1 = "我说你好啊,xxx,我是真的xxx!".to_string();

        let words2 = match_words(&words1, html1);
        assert_eq!(
            vec!["你好啊".to_string(), "xxx".to_string(), "我是真的xxx".to_string()], words2
        );
    }
}