use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub gpio: HashMap<u8, Vec<String>>,
}

#[derive(Deserialize)]
struct RawConfig<'a> {
    #[serde(borrow)]
    gpio: HashMap<&'a str, Vec<String>>,
}

pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Config> {
    let s = fs::read_to_string(path)?;
    from_str(&s)
}

pub fn from_str(s: &str) -> anyhow::Result<Config> {
    let raw: RawConfig = toml::from_str(&s)?;
    Ok(Config {
        gpio: raw
            .gpio
            .into_iter()
            .map(|(pin, command)| match pin.parse() {
                Ok(pin) => Ok((pin, command)),
                Err(e) => Err(e),
            })
            .collect::<Result<_, _>>()?,
    })
}

#[cfg(test)]
mod tests {
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn from_str() {
        let config = assert_ok!(super::from_str(
            r#"
[gpio]
5 = ["inspiro-frame", "next"]
6 = ["uhubctl", "-l", "1-1", "-a", "3"]
"#
        ));
        assert_eq!(
            config,
            super::Config {
                gpio: [
                    (
                        5,
                        ["inspiro-frame", "next"]
                            .iter()
                            .copied()
                            .map(String::from)
                            .collect()
                    ),
                    (
                        6,
                        ["uhubctl", "-l", "1-1", "-a", "3"]
                            .iter()
                            .copied()
                            .map(String::from)
                            .collect()
                    ),
                ]
                .iter()
                .cloned()
                .collect()
            }
        );
        assert_err!(super::from_str(
            r#"
[gpio]
a = ["inspiro-frame", "next"]
"#
        ));
    }
}
