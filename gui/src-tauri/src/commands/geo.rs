#[tauri::command]
pub async fn get_geo_info(ip: Option<String>) -> Result<serde_json::Value, String> {
    let token = std::env::var("IPINFO_TOKEN").map_err(|_| "Token not found".to_string())?;

    let url = match ip {
        Some(addr) => format!("https://api.ipinfo.io/lite/{}?token={}", addr, token),
        None => format!("https://api.ipinfo.io/lite/me?token={}", token),
    };

    let client = reqwest::Client::new();
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;

    let data = res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(data)
}
