use std::io::{self, Stdout};
use systeroid_core::config::Config;
use systeroid_core::error::Result;
use systeroid_core::sysctl::Sysctl;

/// Application controller.
#[derive(Debug)]
pub struct App<'a> {
    /// Sysctl manager.
    sysctl: &'a mut Sysctl,
    /// Configuration.
    config: &'a Config,
    /// Standard output.
    stdout: Stdout,
}

impl<'a> App<'a> {
    /// Constructs a new instance.
    pub fn new(sysctl: &'a mut Sysctl, config: &'a Config) -> Self {
        let stdout = io::stdout();
        Self {
            sysctl,
            config,
            stdout,
        }
    }

    /// Displays all of the available kernel modules.
    pub fn display_parameters(&mut self) -> Result<()> {
        self.sysctl
            .parameters
            .iter()
            .try_for_each(|parameter| parameter.display_value(&self.config.color, &mut self.stdout))
    }

    /// Displays the documentation of a parameter.
    pub fn display_documentation(&mut self, param_name: &str) -> Result<()> {
        if let Some(parameter) = self.sysctl.get_parameter(param_name) {
            parameter.display_documentation(&mut self.stdout)?;
        }
        Ok(())
    }

    /// Updates the parameter if it has the format `name=value`, displays it otherwise.
    pub fn process_parameter(&mut self, mut param_name: String) -> Result<()> {
        let new_value = if param_name.contains('=') {
            let fields = param_name
                .split('=')
                .take(2)
                .map(String::from)
                .collect::<Vec<String>>();
            param_name = fields[0].to_string();
            Some(fields[1].to_string())
        } else {
            None
        };
        if let Some(parameter) = self.sysctl.get_parameter(&param_name) {
            if let Some(new_value) = new_value {
                parameter.update_value(&new_value, &self.config.color, &mut self.stdout)?;
            } else {
                parameter.display_value(&self.config.color, &mut self.stdout)?;
            }
        }
        Ok(())
    }
}