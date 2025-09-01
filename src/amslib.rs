use chrono::DateTime;
use chrono::Local;
use prettytable::Cell;
use prettytable::Row;
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

pub(crate) async fn run(num_adverts: &String, to_date: &DateTime<Local>) -> Result<AmsData, Error> {
    let json_res = post_data(&num_adverts, &to_date).await?;
    let ads = serde_json::from_str(&json_res).expect("Failed to parse json string");
    display_pretty_table(&ads);
    Ok(ads)
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
        // Check if we should indicate that the ad is within the last 3 days.
        let now = Local::now();
        let days_ago = now.signed_duration_since(date);

        table.add_row(Row::new(vec![
            Cell::new(&id.to_string()).style_spec("FY"),
            Cell::new(&date.format("%d/%m").to_string()),
            Cell::new(&place),
            Cell::new(&company),
            // Use the days_ago and the days_new in # of days to mark ads
            match days_ago.num_days() {
                0..3 => Cell::new(&title).style_spec("FG"),
                _ => Cell::new(&title),
            },
        ]));
    }

    table.printstd();
}

fn parse_date(date: String) -> DateTime<Local> {
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
    // Shadow res and check status before processing
    let res = res.error_for_status()?;
    let json = res.text().await?;

    Ok(json)
}
