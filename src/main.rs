use chrono::{Duration, Local, TimeZone};
use config::Config;
use csv;
use error_chain::error_chain;
use jwglxt::STU;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;

mod config;
mod jwglxt;

error_chain!(
    foreign_links {
        SerdeJsonError(serde_json::Error);
        IoError(std::io::Error);
        CsvError(csv::Error);
    }
);

#[derive(Deserialize, Serialize, Debug)]
struct Class {
    kcmc: String,
    xm: String,
    cdmc: String,
    jcs: String,
    zcd: String,
    xqj: String,
}

impl Class {
    fn empty() -> Class {
        Class {
            kcmc: String::new(),
            xm: String::new(),
            cdmc: String::new(),
            jcs: String::new(),
            zcd: String::new(),
            xqj: String::new(),
        }
    }

    fn to_records(&self) -> Vec<Vec<String>> {
        const CLASS_LIST: [[&str; 2]; 8] = [
            ["8:30", "9:15"],
            ["9:25", "10:05"],
            ["10:25", "11:10"],
            ["11:20", "12:00"],
            ["14:30", "15:15"],
            ["15:25", "16:05"],
            ["16:25", "17:10"],
            ["17:20", "18:00"],
        ];
        let mut records = Vec::new();
        let start = Local.ymd(2022, 2, 21);
        let place = self
            .cdmc
            .replace("莲4号教学楼", "4")
            .replace("文科组团楼", "3");

        let time = self
            .jcs
            .split("-")
            .map(|v| v.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let weeks = self
            .zcd
            .replace("周", "")
            .split("-")
            .map(|v| v.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let day = self.xqj.parse::<i64>().unwrap();
        for week in weeks[0]..weeks[1] + 1 {
            let date = (start + Duration::weeks(week - 1) + Duration::days(day - 1))
                .format("%m/%d/%Y")
                .to_string();

            let start_time = CLASS_LIST[time[0] as usize - 1][0];
            let end_time = CLASS_LIST[time[1] as usize - 1][1];
            let record = vec![
                self.kcmc.clone(),
                date.clone(),
                date,
                start_time.to_string(),
                end_time.to_string(),
                "False".to_string(),
                self.xm.clone(),
                place.clone(),
                "False".to_string(),
            ];
            records.push(record);
        }
        records
    }
}

fn get_csv(schedules: &str) -> Result<()> {
    let file = File::create("schedules.csv")?;
    let mut writer = csv::Writer::from_writer(file);
    writer.write_record(&[
        "Subject",
        "Start Date",
        "End Date",
        "Start Time",
        "End Time",
        "All Day Event",
        "Description",
        "Location",
        "Private",
    ])?;
    let schedules: Value = serde_json::from_str(schedules)?;
    for v in schedules["kbList"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            serde_json::from_value(v.clone()).unwrap_or_else(|e| {
                eprintln!("{}", e);
                Class::empty()
            })
        })
        .map(|v| v.to_records())
        .flatten()
    {
        writer.write_record(&v)?;
    }
    const CHINESE_NUM: [&str; 11] = [
        "零", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
    ];
    let start = Local.ymd(2022, 2, 21);
    for i in 1..19 {
        writer.write_record(&[
            format!("第{}周", {
                if i <= 10 {
                    format!("{}", CHINESE_NUM[i as usize])
                } else {
                    format!("{}{}", CHINESE_NUM[10], CHINESE_NUM[i as usize - 10])
                }
            }),
            (start + Duration::weeks(i - 1))
                .format("%m/%d/%Y")
                .to_string(),
            (start + Duration::weeks(i)).format("%m/%d/%Y").to_string(),
            String::new(),
            String::new(),
            String::from("True"),
            String::new(),
            String::new(),
            String::from("False"),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let config = Config::parse();
    let stu = STU::new(config);
    if let Err(e) = stu.login().await {
        println!("Login Error: {}", e);
        return;
    }

    let schedules = match stu.get_schedules(2021, 2).await {
        Ok(v) => v,
        Err(e) => {
            println!("Get Schedule Error: {}", e);
            return;
        }
    };

    if let Err(e) = get_csv(&schedules) {
        println!("CSV Error: {}", e);
        return;
    }
    println!("Done!");
}
