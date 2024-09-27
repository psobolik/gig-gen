const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

pub(crate) fn get_template_names() -> Result<Vec<String>, minreq::Error> {
    let url = format!("{API_URL}/list");
    let response = minreq::get(&url).send()?;
    let mut vec = Vec::new();
    for lines in response.as_str()?.split('\n') {
        for template in lines.split(',') {
            vec.push(template.to_string());
        }
    }
    Ok(vec)
}

pub(crate) fn get_template(template_names: &[String]) -> Result<String, minreq::Error> {
    let url = format!("{API_URL}/{}", template_names.join(","));
    Ok(minreq::get(url).send()?.as_str()?.to_string())
}
