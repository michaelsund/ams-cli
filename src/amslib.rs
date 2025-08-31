use chrono::Local;
use chrono::{DateTime, Utc};
use prettytable::Table;
use reqwest::Client;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmsDataItem {
    pub id: String,
    pub internal_id: Option<usize>,
    pub published_date: String,
    pub title: String,
    pub workplace: String,
    pub workplace_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct AmsData {
    pub ads: Vec<AmsDataItem>,
}

pub(crate) async fn run(num_adverts: &String, to_date: &DateTime<Local>) -> AmsData {
    let json_res = post_data(&num_adverts, &to_date).await;
    let ads: AmsData = serde_json::from_str(&json_res.unwrap()).unwrap();
    display_pretty_table(&ads);
    ads
}

fn display_pretty_table(data: &AmsData) {
    let mut table = Table::new();

    table.add_row(row![FY => "Id", "Date", "Place", "Company", "Title"]);
    for (i, ad) in data.ads.iter().enumerate() {
        let id = i + 1;
        let date = parse_date(ad.published_date.clone());
        let place = &ad.workplace;
        let company = &ad.workplace_name;
        let title = &ad.title;
        table.add_row(row![
            FY -> &id.to_string(),
            &date.format("%d/%m"),
            &place,
            &company,
            &title
        ]);
    }

    table.printstd();
}

fn parse_date(date: String) -> DateTime<Utc> {
    date.parse().expect("Failed to parse datetime")
}

async fn post_data(num_adverts: &String, to_date: &DateTime<Local>) -> Result<String, Error> {
    let client = Client::new();
    let res = client
        .post("https://platsbanken-api.arbetsformedlingen.se/jobs/v1/search")
        .json(&json!({
            "filters": [
                { "type": "occupationField", "value": "apaJ_2ja_LuF" },
                { "type": "region", "value": "xTCk_nT5_Zjm" }
            ],
            "fromDate": "",
            "order": "relevance",
            "maxRecords": &num_adverts,
            "startIndex": 0,
            "toDate": &to_date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            "source": "pb"
        }))
        .send()
        .await?;
    let json = res.text().await.unwrap();

    Ok(json)
}
