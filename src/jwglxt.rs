use anyhow::{anyhow, bail, Result};
use rand::rngs::OsRng;
use reqwest::blocking::Client;
use rsa::{BigUint, PaddingScheme, PublicKey, RsaPublicKey};
use serde_json::Value;

use crate::config::Config;

pub struct Stu {
    username: String,
    password: String,
    client: Client,
}

impl Stu {
    pub fn new(config: Config) -> Stu {
        Stu {
            username: config.username,
            password: config.password,
            client: Client::builder()
                .cookie_store(true)
                .build()
                .unwrap_or_else(|_| panic!("Failed to create client")),
        }
    }

    fn rsa_encode(pwd: &str, n: &str, e: &str) -> Result<String> {
        let rsa_n = BigUint::from_bytes_be(&base64::decode(n)?);
        let rsa_e = BigUint::from_bytes_be(&base64::decode(e)?);
        let key = RsaPublicKey::new(rsa_n, rsa_e)?;
        Ok(base64::encode(&key.encrypt(
            &mut OsRng,
            PaddingScheme::new_pkcs1v15_encrypt(),
            pwd.as_bytes(),
        )?))
    }

    fn get_csrftoken(&self) -> Result<String> {
        let res = self
            .client
            .get("https://jwglxt.haut.edu.cn/jwglxt/xtgl/login_slogin.html")
            .send()?
            .text()?;
        let csrftoken = res
            .split(r#"name="csrftoken" value=""#)
            .nth(1)
            .ok_or_else(|| anyhow!("Failed to get csrftoken"))?
            .split('"')
            .next()
            .ok_or_else(|| anyhow!("Failed to get csrftoken"))?;
        Ok(String::from(csrftoken))
    }

    pub fn login(&self) -> Result<()> {
        let csrftoken = self.get_csrftoken()?;
        let res = self
            .client
            .get("https://jwglxt.haut.edu.cn/jwglxt/xtgl/login_getPublicKey.html")
            .send()?
            .text()?;
        let parsed: Value = serde_json::from_str(&res)?;
        let n = parsed["modulus"]
            .as_str()
            .ok_or_else(|| anyhow!("modulus not found"))?;
        let e = parsed["exponent"]
            .as_str()
            .ok_or_else(|| anyhow!("exponent not found"))?;

        let res = self
            .client
            .post("https://jwglxt.haut.edu.cn/jwglxt/xtgl/login_slogin.html")
            .form(&[
                ("csrftoken", &csrftoken),
                ("yhm", &self.username),
                ("mm", &Stu::rsa_encode(&self.password, n, e)?),
                ("mm", &Stu::rsa_encode(&self.password, n, e)?),
            ])
            .send()?
            .text()?;
        if res.contains("修改密码") {
            Ok(())
        } else {
            bail!("Wrong password or username")
        }
    }

    pub fn get_schedules(&self, xnm: u32, term: u32) -> Result<String> {
        let xqm = match term {
            1 => "3",
            2 => "12",
            _ => bail!("term must be 1 or 2"),
        };
        Ok(self
            .client
            .post("https://jwglxt.haut.edu.cn/jwglxt/kbcx/xskbcx_cxXsKb.html?gnmkdm=N2151")
            .form(&[("xnm", xnm.to_string().as_str()), ("xqm", xqm)])
            .send()?
            .text()?)
    }
}
