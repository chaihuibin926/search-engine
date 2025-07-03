

//TO links 去重 （布隆过滤器）

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

fn get_html_by_address(url: &str, links: &mut Vec<String>) {
    let mut result = url.split("/");
    let protocol = result.next().unwrap();
    result.next().unwrap();
    let domain = result.next().unwrap();

    let html = reqwest::blocking::get(url).unwrap().text().unwrap();

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
   
}

fn main() {
    let mut links = vec![];
    get_html_by_address("https://chaihuibin.vercel.app", &mut links);
    let iter_links = links.clone();
    for i in 0..iter_links.len() {
        get_html_by_address(&iter_links[i], &mut links);
    }
    for i in 0..links.len() {
        println!("{}: {}", i, links[i])
    }
}