use crate::model::*;

pub fn format_msg(entries: &Vec<Entry>) -> String {
    if entries.len() == 0 {
        return "not found".to_string();
    }

    let mut str = String::new();
    try_add_preview_img(&mut str, &entries);
    for entry in entries {
        // title & HLTB link
        str.push_str(&format!("*{}* [🗗]({})", clean_md(&entry.name), entry.link));

        // steam link if present
        if let Some(steam) = &entry.steam {
            str.push_str(&format!(" [🗗Steam]({})", steam))
        }

        // times & new lines
        str.push_str(&format!("\n{}\n",clean_md(&entry.descr)))
    }

    str
}


fn try_add_preview_img(msg: &mut String, entries: &Vec<Entry>) {
    let img_entry = entries.iter()
        .filter(|e| e.img.len() > 0)
        .next();

    match img_entry {
        None => {}
        Some(img_entry) => {
            let escaped_url = escape_url(&img_entry.img);
            let image_link = format!("[ ]({})", escaped_url);
            msg.push_str(image_link.as_str())
        }
    };
}

fn escape_url(str: &str) -> String {
    str
        .replace("(", "%28")
        .replace(")", "%29")
}

fn clean_md(str: &str) -> String {
    str
        .replace("_", "\\_")
        .replace("*", "\\*")
}
