// Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use std::sync::{OnceLock, RwLock};
use unic_langid::LanguageIdentifier;

const EN_FTL: &str = include_str!("../locales/en.ftl");
const ES_FTL: &str = include_str!("../locales/es.ftl");

pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "es"];
pub const DEFAULT_LANGUAGE: &str = "en";

static I18N_STATE: OnceLock<RwLock<I18nState>> = OnceLock::new();

fn get_ftl_content(lang: &str) -> &'static str {
    match lang {
        "es" => ES_FTL,
        _ => EN_FTL,
    }
}

struct I18nState {
    bundle: FluentBundle<FluentResource>,
    current_lang: String,
}

impl I18nState {
    fn new(lang: &str) -> Self {
        let lang_code = if SUPPORTED_LANGUAGES.contains(&lang) {
            lang
        } else {
            DEFAULT_LANGUAGE
        };
        let ftl_content = get_ftl_content(lang_code);
        let langid: LanguageIdentifier =
            lang_code.parse().unwrap_or_else(|_| "en".parse().unwrap());
        let resource = match FluentResource::try_new(ftl_content.to_string()) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to parse FTL resource for '{}': {:?}", lang_code, e);
                FluentResource::try_new(String::new())
                    .unwrap_or(FluentResource::try_new(" ".to_string()).unwrap())
            }
        };
        let mut bundle = FluentBundle::new_concurrent(vec![langid]);
        bundle.set_use_isolating(false);
        if let Err(e) = bundle.add_resource(resource) {
            log::error!("Failed to add FTL resource for '{}': {:?}", lang_code, e);
        }
        Self {
            bundle,
            current_lang: lang_code.to_string(),
        }
    }

    fn get(&self, key: &str) -> String {
        self.get_with_args(key, None)
    }

    fn get_with_args(&self, key: &str, args: Option<&FluentArgs>) -> String {
        let msg = match self.bundle.get_message(key) {
            Some(m) => m,
            None => {
                log::warn!("Missing translation key: {}", key);
                return key.to_string();
            }
        };
        let pattern = match msg.value() {
            Some(p) => p,
            None => return key.to_string(),
        };
        let mut errors = vec![];
        let result = self.bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            log::warn!("Fluent format errors for '{}': {:?}", key, errors);
        }
        result.to_string()
    }

    fn lang(&self) -> &str {
        &self.current_lang
    }
}

pub fn init(lang: &str) {
    let state = I18nState::new(lang);
    let _ = I18N_STATE.set(RwLock::new(state));
}

pub fn set_language(lang: &str) -> bool {
    if !SUPPORTED_LANGUAGES.contains(&lang) {
        return false;
    }
    if let Some(lock) = I18N_STATE.get()
        && let Ok(mut guard) = lock.write()
    {
        *guard = I18nState::new(lang);
        return true;
    }
    false
}

pub fn current_language() -> String {
    if let Some(lock) = I18N_STATE.get()
        && let Ok(guard) = lock.read()
    {
        guard.lang().to_string()
    } else {
        DEFAULT_LANGUAGE.to_string()
    }
}

pub fn t(key: &str) -> String {
    if let Some(lock) = I18N_STATE.get()
        && let Ok(guard) = lock.read()
    {
        guard.get(key)
    } else {
        key.to_string()
    }
}

pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    if let Some(lock) = I18N_STATE.get()
        && let Ok(guard) = lock.read()
    {
        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, (*v).to_string());
        }
        guard.get_with_args(key, Some(&fluent_args))
    } else {
        key.to_string()
    }
}

pub fn detect_system_language() -> String {
    if let Some(locale) = sys_locale::get_locale() {
        let lang = locale.split('-').next().unwrap_or(DEFAULT_LANGUAGE);
        let lang = lang.split('_').next().unwrap_or(DEFAULT_LANGUAGE);
        if SUPPORTED_LANGUAGES.contains(&lang) {
            return lang.to_string();
        }
    }
    DEFAULT_LANGUAGE.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_translate() {
        init("en");
        let result = t("app-name");
        assert_eq!(result, "DEREVO");
    }

    #[test]
    fn test_language_switch() {
        init("en");
        let switched = set_language("es");
        assert!(switched);
        set_language("en");
    }

    #[test]
    fn test_missing_key_returns_key() {
        init("en");
        let result = t("nonexistent-key-12345");
        assert_eq!(result, "nonexistent-key-12345");
    }

    #[test]
    fn test_unsupported_language_rejected() {
        init("en");
        let switched = set_language("xx");
        assert!(!switched);
    }
}
