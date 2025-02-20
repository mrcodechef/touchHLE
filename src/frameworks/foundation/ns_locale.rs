/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSLocale`.

use super::{ns_array, ns_string};
use crate::objc::{id, objc_classes, ClassExports};
use crate::options::Options;
use crate::Environment;
use std::ffi::CStr;

#[derive(Default)]
pub struct State {
    preferred_languages: Option<id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut State {
        &mut env.framework_state.foundation.ns_locale
    }
}

/// Use `msg_class![env; NSLocale preferredLanguages]` rather than calling this
/// directly, because it may be slow and there is no caching.
fn get_preferred_languages(options: &Options) -> Vec<String> {
    if let Some(ref preferred_languages) = options.preferred_languages {
        log!("The app requested your preferred languages. {:?} will reported based on your --preferred-languages= option.", preferred_languages);
        return preferred_languages.clone();
    }

    // Unfortunately Rust-SDL2 doesn't provide a wrapper for this yet.
    let languages = unsafe {
        let mut languages = Vec::new();
        let locales_raw = sdl2_sys::SDL_GetPreferredLocales();
        if !locales_raw.is_null() {
            for i in 0.. {
                let sdl2_sys::SDL_Locale { language, country } = locales_raw.offset(i).read();
                if language.is_null() && country.is_null() {
                    // Terminator
                    break;
                }

                // The country code is ignored because many iPhone OS games
                // (e.g. Super Monkey Ball and Wolfenstein RPG) don't seem to be
                // able to handle it and fall back to English, so providing it
                // does more harm than good. It's also often unhelpful anyway:
                // on macOS, the country code seems to just be the system
                // region, rather than reflecting a preference for
                // e.g. US vs UK English.
                languages.push(CStr::from_ptr(language).to_str().unwrap().to_string());
            }
            sdl2_sys::SDL_free(locales_raw.cast());
        }
        languages
    };

    if languages.is_empty() {
        let lang = "en".to_string();
        log!("The app requested your preferred languages. No information could be retrieved, so {:?} (English) will be reported.", lang);
        vec![lang]
    } else {
        log!("The app requested your preferred languages. {:?} will reported based on your system language preferences.", languages);
        languages
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSLocale: NSObject

// The documentation isn't clear about what the format of the strings should be,
// but Super Monkey Ball does `isEqualToString:` against "fr", "es", "de", "it"
// and "ja", and its locale detection works properly, so presumably they do not
// usually have region suffixes.
+ (id)preferredLanguages {
    if let Some(existing) = State::get(env).preferred_languages {
        existing
    } else {
        let langs = get_preferred_languages(&env.options);
        let lang_ns_strings = langs.into_iter().map(|lang| ns_string::from_rust_string(env, lang)).collect();
        let new = ns_array::from_vec(env, lang_ns_strings);
        State::get(env).preferred_languages = Some(new);
        new
    }
}

// TODO: constructors, more accessors

@end

};
