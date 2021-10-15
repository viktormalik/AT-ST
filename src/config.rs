extern crate yaml_rust;

use crate::analyses::*;
use crate::TestCase;
use log::warn;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlLoader};

/// Project configuration
/// Contains:
///   - compiler information
///   - list of test cases to evaluate the solutions on
///   - list of source analyses to run on the solutions
///   - list of additional scripts to be run on each solution
/// Typically parsed from a YAML file
#[derive(Default)]
pub struct Config {
    // Solutions information
    pub excluded_dirs: Vec<String>,

    // Basic information
    pub src_file: String,

    // Compiler information
    pub compiler: Option<String>,
    pub c_flags: Option<String>,
    pub ld_flags: Option<String>,

    pub test_cases: Vec<TestCase>,
    pub analyses: Vec<Box<dyn Analyser>>,
    pub scripts: Vec<PathBuf>,
}

/// Configuration error
#[derive(Debug)]
pub struct ConfigError {
    msg: String,
}

impl ConfigError {
    fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }

    fn invalid_field(component: &str, field: &str, expected_type: &str) -> Self {
        Self {
            msg: format!(
                "\'{}\' has invalid value of field \'{}\' ({} expected)",
                component, field, expected_type
            ),
        }
    }

    fn missing_field(component: &str, field: &str) -> Self {
        Self {
            msg: format!(
                "\'{}\' is missing a mandatory field \'{}\'",
                component, field
            ),
        }
    }
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid configuration: {}", self.msg)
    }
}

impl Config {
    pub fn from_yaml(yaml_file: &Path, project_path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut yaml_str = String::new();
        File::open(project_path.join(yaml_file))?.read_to_string(&mut yaml_str)?;
        let yaml = YamlLoader::load_from_str(&yaml_str)?;

        let mut result = Config {
            // Set mandatory fields here
            src_file: mandatory_field_str(&yaml[0], "config", "source")?,
            ..Default::default()
        };

        let config_options = yaml[0]
            .as_hash()
            .ok_or(ConfigError::new("top level entry is not a YAML dict"))?;

        for (key, val) in config_options.iter() {
            match key.as_str() {
                // Optional fields
                Some("solutions") => {
                    check_fields(val, "solutions", &vec!["exclude-dirs"])?;
                    result.excluded_dirs =
                        optional_field_vec_str(val, "solutions", "exclude-dirs")?.unwrap_or(vec![])
                }
                Some("compiler") => {
                    check_fields(val, "compiler", &vec!["CC", "CFLAGS", "LDFLAGS"])?;
                    result.compiler = optional_field_str(val, "compiler", "CC")?;
                    result.c_flags = optional_field_str(val, "compiler", "CFLAGS")?;
                    result.ld_flags = optional_field_str(val, "compiler", "LDFLAGS")?;
                }
                Some("analyses") => result.analyses = analyses_from_yaml(val)?,
                Some("tests") => result.test_cases = tests_from_yaml(val)?,
                Some("scripts") => {
                    result.scripts = optional_field_vec_str(&yaml[0], "config", "scripts")?
                        .unwrap_or(vec![])
                        .iter()
                        .map(|s| project_path.join(s))
                        .collect();
                }
                // Mandatory fields (already set)
                Some("source") => {}
                Some(k) => {
                    warn!("Unsupported config option: {}", k);
                }
                None => {
                    warn!("Invalid config option: {:?}", key);
                }
            };
        }
        Ok(result)
    }
}

fn tests_from_yaml(yaml: &Yaml) -> Result<Vec<TestCase>, ConfigError> {
    match yaml.as_vec() {
        Some(v) => v
            .iter()
            .map(|test| {
                let test_name = optional_field_str(test, "test", "name")?.unwrap_or_default();
                check_fields(
                    test,
                    &test_name,
                    &vec!["name", "score", "args", "stdin", "stdout"],
                )?;
                Ok(TestCase {
                    name: test_name.to_string(),
                    score: mandatory_field_f64(test, &test_name, "score")?,
                    args: optional_field_str(test, &test_name, "args")?
                        .unwrap_or_default()
                        .split_whitespace()
                        .map(String::from)
                        .collect(),
                    stdin: optional_field_str(test, &test_name, "stdin")?,
                    stdout: optional_field_str(test, &test_name, "stdout")?,
                })
            })
            .collect(),
        None => Ok(vec![]),
    }
}

fn analyses_from_yaml(yaml: &Yaml) -> Result<Vec<Box<dyn Analyser>>, ConfigError> {
    let mut result = vec![];
    for analysis in yaml.as_vec().unwrap_or(&vec![]) {
        let analysis_name = mandatory_field_str(analysis, "analysis", "analyser")?;
        let kind = AnalyserKind::from(&analysis_name);
        match &kind {
            AnalyserKind::NoCall => {
                check_analysis_fields(analysis, &analysis_name, &vec!["funs", "penalty"])?;
                result.push(Box::new(NoCallAnalyser::new(
                    mandatory_field_vec_str(analysis, "no-call analyser", "funs")?,
                    mandatory_field_f64(analysis, "no-call analyser", "penalty")?,
                )) as Box<dyn Analyser>);
            }
            AnalyserKind::NoHeader => {
                check_analysis_fields(analysis, &analysis_name, &vec!["header", "penalty"])?;
                result.push(Box::new(NoHeaderAnalyser::new(
                    mandatory_field_str(analysis, "no-header analyser", "header")?,
                    mandatory_field_f64(analysis, "no-header analyser", "penalty")?,
                )) as Box<dyn Analyser>);
            }
            AnalyserKind::NoGlobals => {
                check_analysis_fields(analysis, &analysis_name, &vec!["penalty"])?;
                result.push(Box::new(NoGlobalsAnalyser::new(mandatory_field_f64(
                    analysis,
                    "no-globals",
                    "penalty",
                )?)) as Box<dyn Analyser>);
            }
            AnalyserKind::Unsupported => {
                warn!(
                    "Configuration contains an unsupported analysis \'{}\'",
                    analysis_name
                );
            }
        }
    }
    Ok(result)
}

/// Check if `yaml` is a YAML dictionary (hash) and that it does not contain any keys
/// except those given in `fields`. If an extra key is found, emits a warning.
fn check_fields(yaml: &Yaml, name: &str, fields: &Vec<&str>) -> Result<(), ConfigError> {
    for field in yaml
        .as_hash()
        .ok_or(ConfigError::new(
            format!("\'{}\' has incorrect format", name).as_str(),
        ))?
        .keys()
    {
        let field_name = field.as_str().unwrap_or_default();
        if !fields.contains(&field_name) {
            warn!(
                "Configuration of \'{}\' has unsupported option \'{}\'",
                name, field_name
            );
        }
    }
    Ok(())
}

/// Same as `check_fields`, only specialized for analysis config, which always contains
/// a field "analyser".
fn check_analysis_fields(yaml: &Yaml, name: &str, fields: &Vec<&str>) -> Result<(), ConfigError> {
    let mut analyser_fields = fields.clone();
    analyser_fields.push("analyser");
    let analyser_name = "analyser ".to_string() + name;
    check_fields(yaml, &analyser_name, &analyser_fields)
}

/// Parse `field` from `yaml` as a f64 number.
/// Yields `ConfigError` if the value is not a f64.
/// Returns None if `yaml` does not contain `field`.
fn optional_field_f64(yaml: &Yaml, name: &str, field: &str) -> Result<Option<f64>, ConfigError> {
    match &yaml[field] {
        Yaml::BadValue => Ok(None),
        val => Ok(Some(val.as_f64().ok_or(ConfigError::invalid_field(
            name,
            field,
            "float number",
        ))?)),
    }
}

/// Parse `field` from `yaml` as a f64 number.
/// Yields `ConfigError` if `yaml` does not contain `field` or its value is not a f64.
fn mandatory_field_f64(yaml: &Yaml, name: &str, field: &str) -> Result<f64, ConfigError> {
    optional_field_f64(yaml, name, field)?.ok_or_else(|| ConfigError::missing_field(name, field))
}

/// Parse `field` from `yaml` as a string.
/// Yields `ConfigError` if the value is not a string.
/// Returns None if `yaml` does not contain `field`.
fn optional_field_str(yaml: &Yaml, name: &str, field: &str) -> Result<Option<String>, ConfigError> {
    match &yaml[field] {
        Yaml::BadValue => Ok(None),
        val => {
            Ok(Some(val.as_str().map(String::from).ok_or(
                ConfigError::invalid_field(name, field, "string"),
            )?))
        }
    }
}

/// Parse `field` from `yaml` as a string.
/// Yields `ConfigError` if `yaml` does not contain `field` or its value is not a string.
fn mandatory_field_str(yaml: &Yaml, name: &str, field: &str) -> Result<String, ConfigError> {
    optional_field_str(yaml, name, field)?.ok_or_else(|| ConfigError::missing_field(name, field))
}

/// Parse `field` from `yaml` as a vector of strings.
/// Yields `ConfigError` if the value is not a vector of strings.
/// Returns None if `yaml` does not contain `field`.
fn optional_field_vec_str(
    yaml: &Yaml,
    name: &str,
    field: &str,
) -> Result<Option<Vec<String>>, ConfigError> {
    match &yaml[field] {
        Yaml::BadValue => Ok(None),
        val => Ok(Some(
            val.as_vec()
                .ok_or(ConfigError::invalid_field(name, field, "list of strings"))?
                .iter()
                .map(|s| {
                    s.as_str()
                        .map(String::from)
                        .ok_or(ConfigError::invalid_field(name, field, "list of strings"))
                })
                .collect::<Result<Vec<String>, ConfigError>>()?,
        )),
    }
}

/// Parse `field` from `yaml` as a vector of string.
/// Yields `ConfigError` if `yaml` does not contain `field`
/// or its value is not a vector of strings.
fn mandatory_field_vec_str(
    yaml: &Yaml,
    name: &str,
    field: &str,
) -> Result<Vec<String>, ConfigError> {
    optional_field_vec_str(yaml, name, field)?
        .ok_or_else(|| ConfigError::missing_field(name, field))
}
