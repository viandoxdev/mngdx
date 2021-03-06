// Shoutout to Vim Macros

use std::fmt::Display;

use serde::Deserialize;

#[derive(Hash, PartialEq, Eq, Deserialize, Debug, Clone)]
#[serde(from = "String", into = "String")]
pub enum LanguageCode {
    // OFFICIAL ISO 639-1 CODES
    Afar,          // aa
    Abkhazian,     // ab
    Afrikaans,     // af
    Amharic,       // am
    Arabic,        // ar
    Assamese,      // as
    Aymara,        // ay
    Azerbaijani,   // az
    Bashkir,       // ba
    Byelorussian,  // be
    Bulgarian,     // bg
    Bihari,        // bh
    Bislama,       // bi
    Bengali,       // bn
    Tibetan,       // bo
    Breton,        // br
    Catalan,       // ca
    Corsican,      // co
    Czech,         // cs
    Welch,         // cy
    Danish,        // da
    German,        // de
    Bhutani,       // dz
    Greek,         // el
    English,       // en
    Esperanto,     // eo
    Spanish,       // es
    Estonian,      // et
    Basque,        // eu
    Persian,       // fa
    Finnish,       // fi
    Fiji,          // fj
    Faeroese,      // fo
    French,        // fr
    Frisian,       // fy
    Irish,         // ga
    ScotsGaelic,   // gd
    Galician,      // gl
    Guarani,       // gn
    Gujarati,      // gu
    Hausa,         // ha
    Hindi,         // hi
    Hebrew,        // he
    Croatian,      // hr
    Hungarian,     // hu
    Armenian,      // hy
    Interlingua,   // ia
    Indonesian,    // id
    Interlingue,   // ie
    Inupiak,       // ik
    Icelandic,     // is
    Italian,       // it
    Inuktitut,     // iu
    Japanese,      // ja
    Javanese,      // jw
    Georgian,      // ka
    Kazakh,        // kk
    Greenlandic,   // kl
    Cambodian,     // km
    Kannada,       // kn
    Korean,        // ko
    Kashmiri,      // ks
    Kurdish,       // ku
    Kirghiz,       // ky
    Latin,         // la
    Lingala,       // ln
    Laothian,      // lo
    Lithuanian,    // lt
    Latvian,       // lv
    Malagasy,      // mg
    Maori,         // mi
    Macedonian,    // mk
    Malayalam,     // ml
    Mongolian,     // mn
    Moldavian,     // mo
    Marathi,       // mr
    Malay,         // ms
    Maltese,       // mt
    Burmese,       // my
    Nauru,         // na
    Nepali,        // ne
    Dutch,         // nl
    Norwegian,     // no
    Occitan,       // oc
    Oromo,         // om
    Oriya,         // or
    Punjabi,       // pa
    Polish,        // pl
    Pashto,        // ps
    Portuguese,    // pt
    Quechua,       // qu
    RhaetoRomance, // rm
    Kirundi,       // rn
    Romanian,      // ro
    Russian,       // ru
    Kinyarwanda,   // rw
    Sanskrit,      // sa
    Sindhi,        // sd
    Sangro,        // sg
    SerboCroatian, // sh
    Singhalese,    // si
    Slovak,        // sk
    Slovenian,     // sl
    Samoan,        // sm
    Shona,         // sn
    Somali,        // so
    Albanian,      // sq
    Serbian,       // sr
    Siswati,       // ss
    Sesotho,       // st
    Sudanese,      // su
    Swedish,       // sv
    Swahili,       // sw
    Tamil,         // ta
    Tegulu,        // te
    Tajik,         // tg
    Thai,          // th
    Tigrinya,      // ti
    Turkmen,       // tk
    Setswana,      // tn
    Tonga,         // to
    Turkish,       // tr
    Tsonga,        // ts
    Tatar,         // tt
    Twi,           // tw
    Uigur,         // ug
    Ukrainian,     // uk
    Urdu,          // ur
    Uzbek,         // uz
    Vietnamese,    // vi
    Volapuk,       // vo
    Wolof,         // wo
    Xhosa,         // xh
    Yiddish,       // yi
    Yoruba,        // yo
    Zhuang,        // za
    Chinese,       // zh
    Zulu,          // zu

    // UNUSED (because mangadex has weird lang codes)
    Tagalog, // tl (code taken by Filipino)

    // MANGADEX SPECIFIC
    TraditionalChinese,   // zh-hk
    BrazilianPortugese,   // pt-br
    LatinAmericanSpanish, // es-la
    RomanizedJapanese,    // ja-ro
    RomanizedKorean,      // ko-ro
    RomanizedChinese,     // zh-ro

    Filipino, // tl

    // Only on mangadex, the null language !
    Null,

    // OTHERS (just in case)
    Any(String),
}

impl From<LanguageCode> for String {
    fn from(lang: LanguageCode) -> String {
        match lang {
            LanguageCode::Afar => "aa".to_owned(),
            LanguageCode::Abkhazian => "ab".to_owned(),
            LanguageCode::Afrikaans => "af".to_owned(),
            LanguageCode::Amharic => "am".to_owned(),
            LanguageCode::Arabic => "ar".to_owned(),
            LanguageCode::Assamese => "as".to_owned(),
            LanguageCode::Aymara => "ay".to_owned(),
            LanguageCode::Azerbaijani => "az".to_owned(),
            LanguageCode::Bashkir => "ba".to_owned(),
            LanguageCode::Byelorussian => "be".to_owned(),
            LanguageCode::Bulgarian => "bg".to_owned(),
            LanguageCode::Bihari => "bh".to_owned(),
            LanguageCode::Bislama => "bi".to_owned(),
            LanguageCode::Bengali => "bn".to_owned(),
            LanguageCode::Tibetan => "bo".to_owned(),
            LanguageCode::Breton => "br".to_owned(),
            LanguageCode::Catalan => "ca".to_owned(),
            LanguageCode::Corsican => "co".to_owned(),
            LanguageCode::Czech => "cs".to_owned(),
            LanguageCode::Welch => "cy".to_owned(),
            LanguageCode::Danish => "da".to_owned(),
            LanguageCode::German => "de".to_owned(),
            LanguageCode::Bhutani => "dz".to_owned(),
            LanguageCode::Greek => "el".to_owned(),
            LanguageCode::English => "en".to_owned(),
            LanguageCode::Esperanto => "eo".to_owned(),
            LanguageCode::Spanish => "es".to_owned(),
            LanguageCode::Estonian => "et".to_owned(),
            LanguageCode::Basque => "eu".to_owned(),
            LanguageCode::Persian => "fa".to_owned(),
            LanguageCode::Finnish => "fi".to_owned(),
            LanguageCode::Fiji => "fj".to_owned(),
            LanguageCode::Faeroese => "fo".to_owned(),
            LanguageCode::French => "fr".to_owned(),
            LanguageCode::Frisian => "fy".to_owned(),
            LanguageCode::Irish => "ga".to_owned(),
            LanguageCode::ScotsGaelic => "gd".to_owned(),
            LanguageCode::Galician => "gl".to_owned(),
            LanguageCode::Guarani => "gn".to_owned(),
            LanguageCode::Gujarati => "gu".to_owned(),
            LanguageCode::Hausa => "ha".to_owned(),
            LanguageCode::Hindi => "hi".to_owned(),
            LanguageCode::Hebrew => "he".to_owned(),
            LanguageCode::Croatian => "hr".to_owned(),
            LanguageCode::Hungarian => "hu".to_owned(),
            LanguageCode::Armenian => "hy".to_owned(),
            LanguageCode::Interlingua => "ia".to_owned(),
            LanguageCode::Indonesian => "id".to_owned(),
            LanguageCode::Interlingue => "ie".to_owned(),
            LanguageCode::Inupiak => "ik".to_owned(),
            LanguageCode::Icelandic => "is".to_owned(),
            LanguageCode::Italian => "it".to_owned(),
            LanguageCode::Inuktitut => "iu".to_owned(),
            LanguageCode::Japanese => "ja".to_owned(),
            LanguageCode::Javanese => "jw".to_owned(),
            LanguageCode::Georgian => "ka".to_owned(),
            LanguageCode::Kazakh => "kk".to_owned(),
            LanguageCode::Greenlandic => "kl".to_owned(),
            LanguageCode::Cambodian => "km".to_owned(),
            LanguageCode::Kannada => "kn".to_owned(),
            LanguageCode::Korean => "ko".to_owned(),
            LanguageCode::Kashmiri => "ks".to_owned(),
            LanguageCode::Kurdish => "ku".to_owned(),
            LanguageCode::Kirghiz => "ky".to_owned(),
            LanguageCode::Latin => "la".to_owned(),
            LanguageCode::Lingala => "ln".to_owned(),
            LanguageCode::Laothian => "lo".to_owned(),
            LanguageCode::Lithuanian => "lt".to_owned(),
            LanguageCode::Latvian => "lv".to_owned(),
            LanguageCode::Malagasy => "mg".to_owned(),
            LanguageCode::Maori => "mi".to_owned(),
            LanguageCode::Macedonian => "mk".to_owned(),
            LanguageCode::Malayalam => "ml".to_owned(),
            LanguageCode::Mongolian => "mn".to_owned(),
            LanguageCode::Moldavian => "mo".to_owned(),
            LanguageCode::Marathi => "mr".to_owned(),
            LanguageCode::Malay => "ms".to_owned(),
            LanguageCode::Maltese => "mt".to_owned(),
            LanguageCode::Burmese => "my".to_owned(),
            LanguageCode::Nauru => "na".to_owned(),
            LanguageCode::Nepali => "ne".to_owned(),
            LanguageCode::Dutch => "nl".to_owned(),
            LanguageCode::Norwegian => "no".to_owned(),
            LanguageCode::Occitan => "oc".to_owned(),
            LanguageCode::Oromo => "om".to_owned(),
            LanguageCode::Oriya => "or".to_owned(),
            LanguageCode::Punjabi => "pa".to_owned(),
            LanguageCode::Polish => "pl".to_owned(),
            LanguageCode::Pashto => "ps".to_owned(),
            LanguageCode::Portuguese => "pt".to_owned(),
            LanguageCode::Quechua => "qu".to_owned(),
            LanguageCode::RhaetoRomance => "rm".to_owned(),
            LanguageCode::Kirundi => "rn".to_owned(),
            LanguageCode::Romanian => "ro".to_owned(),
            LanguageCode::Russian => "ru".to_owned(),
            LanguageCode::Kinyarwanda => "rw".to_owned(),
            LanguageCode::Sanskrit => "sa".to_owned(),
            LanguageCode::Sindhi => "sd".to_owned(),
            LanguageCode::Sangro => "sg".to_owned(),
            LanguageCode::SerboCroatian => "sh".to_owned(),
            LanguageCode::Singhalese => "si".to_owned(),
            LanguageCode::Slovak => "sk".to_owned(),
            LanguageCode::Slovenian => "sl".to_owned(),
            LanguageCode::Samoan => "sm".to_owned(),
            LanguageCode::Shona => "sn".to_owned(),
            LanguageCode::Somali => "so".to_owned(),
            LanguageCode::Albanian => "sq".to_owned(),
            LanguageCode::Serbian => "sr".to_owned(),
            LanguageCode::Siswati => "ss".to_owned(),
            LanguageCode::Sesotho => "st".to_owned(),
            LanguageCode::Sudanese => "su".to_owned(),
            LanguageCode::Swedish => "sv".to_owned(),
            LanguageCode::Swahili => "sw".to_owned(),
            LanguageCode::Tamil => "ta".to_owned(),
            LanguageCode::Tegulu => "te".to_owned(),
            LanguageCode::Tajik => "tg".to_owned(),
            LanguageCode::Thai => "th".to_owned(),
            LanguageCode::Tigrinya => "ti".to_owned(),
            LanguageCode::Turkmen => "tk".to_owned(),
            LanguageCode::Tagalog => "tl".to_owned(),
            LanguageCode::Setswana => "tn".to_owned(),
            LanguageCode::Tonga => "to".to_owned(),
            LanguageCode::Turkish => "tr".to_owned(),
            LanguageCode::Tsonga => "ts".to_owned(),
            LanguageCode::Tatar => "tt".to_owned(),
            LanguageCode::Twi => "tw".to_owned(),
            LanguageCode::Uigur => "ug".to_owned(),
            LanguageCode::Ukrainian => "uk".to_owned(),
            LanguageCode::Urdu => "ur".to_owned(),
            LanguageCode::Uzbek => "uz".to_owned(),
            LanguageCode::Vietnamese => "vi".to_owned(),
            LanguageCode::Volapuk => "vo".to_owned(),
            LanguageCode::Wolof => "wo".to_owned(),
            LanguageCode::Xhosa => "xh".to_owned(),
            LanguageCode::Yiddish => "yi".to_owned(),
            LanguageCode::Yoruba => "yo".to_owned(),
            LanguageCode::Zhuang => "za".to_owned(),
            LanguageCode::Chinese => "zh".to_owned(),
            LanguageCode::Zulu => "zu".to_owned(),

            LanguageCode::TraditionalChinese => "zh-hk".to_owned(),
            LanguageCode::BrazilianPortugese => "pt-br".to_owned(),
            LanguageCode::LatinAmericanSpanish => "es-la".to_owned(),
            LanguageCode::RomanizedJapanese => "ja-ro".to_owned(),
            LanguageCode::RomanizedKorean => "ko-ro".to_owned(),
            LanguageCode::RomanizedChinese => "zh-ro".to_owned(),

            LanguageCode::Filipino => "tl".to_owned(),

            LanguageCode::Null => "NULL".to_owned(),

            LanguageCode::Any(s) => s,
        }
    }
}

impl From<String> for LanguageCode {
    fn from(v: String) -> Self {
        match v.as_str() {
            "aa" => LanguageCode::Afar,
            "ab" => LanguageCode::Abkhazian,
            "af" => LanguageCode::Afrikaans,
            "am" => LanguageCode::Amharic,
            "ar" => LanguageCode::Arabic,
            "as" => LanguageCode::Assamese,
            "ay" => LanguageCode::Aymara,
            "az" => LanguageCode::Azerbaijani,
            "ba" => LanguageCode::Bashkir,
            "be" => LanguageCode::Byelorussian,
            "bg" => LanguageCode::Bulgarian,
            "bh" => LanguageCode::Bihari,
            "bi" => LanguageCode::Bislama,
            "bn" => LanguageCode::Bengali,
            "bo" => LanguageCode::Tibetan,
            "br" => LanguageCode::Breton,
            "ca" => LanguageCode::Catalan,
            "co" => LanguageCode::Corsican,
            "cs" => LanguageCode::Czech,
            "cy" => LanguageCode::Welch,
            "da" => LanguageCode::Danish,
            "de" => LanguageCode::German,
            "dz" => LanguageCode::Bhutani,
            "el" => LanguageCode::Greek,
            "en" => LanguageCode::English,
            "eo" => LanguageCode::Esperanto,
            "es" => LanguageCode::Spanish,
            "et" => LanguageCode::Estonian,
            "eu" => LanguageCode::Basque,
            "fa" => LanguageCode::Persian,
            "fi" => LanguageCode::Finnish,
            "fj" => LanguageCode::Fiji,
            "fo" => LanguageCode::Faeroese,
            "fr" => LanguageCode::French,
            "fy" => LanguageCode::Frisian,
            "ga" => LanguageCode::Irish,
            "gd" => LanguageCode::ScotsGaelic,
            "gl" => LanguageCode::Galician,
            "gn" => LanguageCode::Guarani,
            "gu" => LanguageCode::Gujarati,
            "ha" => LanguageCode::Hausa,
            "hi" => LanguageCode::Hindi,
            "he" => LanguageCode::Hebrew,
            "hr" => LanguageCode::Croatian,
            "hu" => LanguageCode::Hungarian,
            "hy" => LanguageCode::Armenian,
            "ia" => LanguageCode::Interlingua,
            "id" => LanguageCode::Indonesian,
            "ie" => LanguageCode::Interlingue,
            "ik" => LanguageCode::Inupiak,
            "is" => LanguageCode::Icelandic,
            "it" => LanguageCode::Italian,
            "iu" => LanguageCode::Inuktitut,
            "ja" => LanguageCode::Japanese,
            "jw" => LanguageCode::Javanese,
            "ka" => LanguageCode::Georgian,
            "kk" => LanguageCode::Kazakh,
            "kl" => LanguageCode::Greenlandic,
            "km" => LanguageCode::Cambodian,
            "kn" => LanguageCode::Kannada,
            "ko" => LanguageCode::Korean,
            "ks" => LanguageCode::Kashmiri,
            "ku" => LanguageCode::Kurdish,
            "ky" => LanguageCode::Kirghiz,
            "la" => LanguageCode::Latin,
            "ln" => LanguageCode::Lingala,
            "lo" => LanguageCode::Laothian,
            "lt" => LanguageCode::Lithuanian,
            "lv" => LanguageCode::Latvian,
            "mg" => LanguageCode::Malagasy,
            "mi" => LanguageCode::Maori,
            "mk" => LanguageCode::Macedonian,
            "ml" => LanguageCode::Malayalam,
            "mn" => LanguageCode::Mongolian,
            "mo" => LanguageCode::Moldavian,
            "mr" => LanguageCode::Marathi,
            "ms" => LanguageCode::Malay,
            "mt" => LanguageCode::Maltese,
            "my" => LanguageCode::Burmese,
            "na" => LanguageCode::Nauru,
            "ne" => LanguageCode::Nepali,
            "nl" => LanguageCode::Dutch,
            "no" => LanguageCode::Norwegian,
            "oc" => LanguageCode::Occitan,
            "om" => LanguageCode::Oromo,
            "or" => LanguageCode::Oriya,
            "pa" => LanguageCode::Punjabi,
            "pl" => LanguageCode::Polish,
            "ps" => LanguageCode::Pashto,
            "pt" => LanguageCode::Portuguese,
            "qu" => LanguageCode::Quechua,
            "rm" => LanguageCode::RhaetoRomance,
            "rn" => LanguageCode::Kirundi,
            "ro" => LanguageCode::Romanian,
            "ru" => LanguageCode::Russian,
            "rw" => LanguageCode::Kinyarwanda,
            "sa" => LanguageCode::Sanskrit,
            "sd" => LanguageCode::Sindhi,
            "sg" => LanguageCode::Sangro,
            "sh" => LanguageCode::SerboCroatian,
            "si" => LanguageCode::Singhalese,
            "sk" => LanguageCode::Slovak,
            "sl" => LanguageCode::Slovenian,
            "sm" => LanguageCode::Samoan,
            "sn" => LanguageCode::Shona,
            "so" => LanguageCode::Somali,
            "sq" => LanguageCode::Albanian,
            "sr" => LanguageCode::Serbian,
            "ss" => LanguageCode::Siswati,
            "st" => LanguageCode::Sesotho,
            "su" => LanguageCode::Sudanese,
            "sv" => LanguageCode::Swedish,
            "sw" => LanguageCode::Swahili,
            "ta" => LanguageCode::Tamil,
            "te" => LanguageCode::Tegulu,
            "tg" => LanguageCode::Tajik,
            "th" => LanguageCode::Thai,
            "ti" => LanguageCode::Tigrinya,
            "tk" => LanguageCode::Turkmen,
            "tl" => LanguageCode::Filipino, // Replaces Tagalog
            "tn" => LanguageCode::Setswana,
            "to" => LanguageCode::Tonga,
            "tr" => LanguageCode::Turkish,
            "ts" => LanguageCode::Tsonga,
            "tt" => LanguageCode::Tatar,
            "tw" => LanguageCode::Twi,
            "ug" => LanguageCode::Uigur,
            "uk" => LanguageCode::Ukrainian,
            "ur" => LanguageCode::Urdu,
            "uz" => LanguageCode::Uzbek,
            "vi" => LanguageCode::Vietnamese,
            "vo" => LanguageCode::Volapuk,
            "wo" => LanguageCode::Wolof,
            "xh" => LanguageCode::Xhosa,
            "yi" => LanguageCode::Yiddish,
            "yo" => LanguageCode::Yoruba,
            "za" => LanguageCode::Zhuang,
            "zh" => LanguageCode::Chinese,
            "zu" => LanguageCode::Zulu,

            "zh-hk" => LanguageCode::TraditionalChinese,
            "pt-br" => LanguageCode::BrazilianPortugese,
            "es-la" => LanguageCode::LatinAmericanSpanish,
            "ja-ro" => LanguageCode::RomanizedJapanese,
            "ko-ro" => LanguageCode::RomanizedKorean,
            "zh-ro" => LanguageCode::RomanizedChinese,

            "NULL" => {
                log::debug!("NULL language found. (This is a problem with mangadex)");
                LanguageCode::Null
            }

            _ => {
                // error because if that happens, it means I've forgotten a code, and need to add
                // it.
                log::error!("Unknown language code ({})", v);
                LanguageCode::Any(v)
            }
        }
    }
}
impl Display for LanguageCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <Self as Into::<String>>::into(self.clone()))
    }
}
