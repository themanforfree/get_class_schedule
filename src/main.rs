use anyhow::Result;
use chrono::{Duration, Local, TimeZone};
use config::Config;
use jwglxt::Stu;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;

mod config;
mod jwglxt;

#[derive(Deserialize, Serialize, Debug)]
struct Class {
    kcmc: String,
    xm: String,
    cdmc: String,
    jcs: String,
    zcd: String,
    xqj: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Record {
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "Start Date")]
    start_date: String,
    #[serde(rename = "End Date")]
    end_date: String,
    #[serde(rename = "Start Time")]
    start_time: String,
    #[serde(rename = "End Time")]
    end_time: String,
    #[serde(rename = "All Day Event")]
    all_day: bool,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Location")]
    location: String,
    #[serde(rename = "Private")]
    private: bool,
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

    fn to_records(&self) -> Vec<Record> {
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
            .split('-')
            .map(|v| {
                v.parse::<u32>().unwrap_or_else(|e| {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1)
                })
            })
            .collect::<Vec<_>>();
        let weeks = self
            .zcd
            .replace('周', "")
            .split('-')
            .map(|v| {
                v.parse::<i64>().unwrap_or_else(|e| {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1)
                })
            })
            .collect::<Vec<_>>();
        let day = self.xqj.parse::<i64>().unwrap_or_else(|e| {
            eprintln!("Parse error: {}", e);
            std::process::exit(1)
        });
        for week in weeks[0]..weeks[1] + 1 {
            let date = (start + Duration::weeks(week - 1) + Duration::days(day - 1))
                .format("%m/%d/%Y")
                .to_string();

            let start_time = CLASS_LIST[time[0] as usize - 1][0];
            let end_time = CLASS_LIST[time[1] as usize - 1][1];
            let record = Record {
                subject: self.kcmc.clone(),
                start_date: date.clone(),
                end_date: date.clone(),
                start_time: start_time.to_string(),
                end_time: end_time.to_string(),
                all_day: false,
                description: self.xm.clone(),
                location: place.clone(),
                private: false,
            };
            records.push(record);
        }
        records
    }
}

fn get_csv(schedules: &str) -> Result<()> {
    let file = File::create("schedules.csv")?;
    let mut writer = csv::Writer::from_writer(file);
    let schedules: Value = serde_json::from_str(schedules)?;
    for record in schedules["kbList"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("kbList is not an array"))?
        .iter()
        .map(|v| {
            serde_json::from_value(v.clone()).unwrap_or_else(|e| {
                eprintln!("{}", e);
                Class::empty()
            })
        })
        .flat_map(|v| v.to_records())
    {
        writer.serialize(record)?;
    }
    const CHINESE_NUM: [&str; 11] = [
        "零", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
    ];
    let start = Local.ymd(2022, 2, 21);
    for i in 1..19 {
        writer.serialize(Record {
            subject: format!("第{}周", {
                if i <= 10 {
                    CHINESE_NUM[i as usize].to_string()
                } else {
                    format!("{}{}", CHINESE_NUM[10], CHINESE_NUM[i as usize - 10])
                }
            }),
            start_date: (start + Duration::weeks(i - 1))
                .format("%m/%d/%Y")
                .to_string(),
            end_date: (start + Duration::weeks(i)).format("%m/%d/%Y").to_string(),
            start_time: String::new(),
            end_time: String::new(),
            all_day: true,
            description: String::new(),
            location: String::new(),
            private: false,
        })?;
    }
    writer.flush()?;
    Ok(())
}

fn main() {
    let config = Config::parse();
    let stu = Stu::new(config);
    if let Err(e) = stu.login() {
        println!("Login Error: {}", e);
        return;
    }

    let schedules = match stu.get_schedules(2021, 2) {
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
