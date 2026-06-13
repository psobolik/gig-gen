const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

pub(crate) fn get_template_names() -> Result<Vec<String>, crate::Error> {
    let url = format!("{API_URL}/list");
    let response = minreq::get(&url).send()?;
    if response.status_code != 200 {
        Err(crate::ApiError::new(response.status_code, &response.reason_phrase).into())
    } else {
        let mut vec = Vec::new();
        for lines in response.as_str()?.split('\n') {
            for template in lines.split(',') {
                vec.push(template.to_string());
            }
        }
        Ok(vec)
    }
}

pub(crate) fn get_template(template_names: &[String]) -> Result<String, crate::Error> {
    let url = format!("{API_URL}/{}", template_names.join(","));
    let response = minreq::get(&url).send()?;
    if response.status_code != 200 {
        Err(crate::ApiError::new(response.status_code, &response.reason_phrase).into())
    } else {
        Ok(response.as_str()?.to_string())
    }
}
