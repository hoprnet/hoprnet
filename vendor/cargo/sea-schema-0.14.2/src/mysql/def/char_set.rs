#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use crate as sea_schema;

/// Ref: https://dev.mysql.com/doc/refman/8.0/en/charset-charsets.html
#[derive(Clone, Debug, PartialEq, sea_query::Iden, sea_schema_derive::Name)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[catch = "string_to_unknown"]
pub enum CharSet {
    #[iden = "armscii8"]
    Armscii8,
    #[iden = "ascii"]
    Ascii,
    #[iden = "big5"]
    Big5,
    #[iden = "binary"]
    Binary,
    #[iden = "cp1250"]
    Cp1250,
    #[iden = "cp1251"]
    Cp1251,
    #[iden = "cp1256"]
    Cp1256,
    #[iden = "cp1257"]
    Cp1257,
    #[iden = "cp850"]
    Cp850,
    #[iden = "cp852"]
    Cp852,
    #[iden = "cp866"]
    Cp866,
    #[iden = "cp932"]
    Cp932,
    #[iden = "dec8"]
    Dec8,
    #[iden = "eucjpms"]
    Eucjpms,
    #[iden = "euckr"]
    Euckr,
    #[iden = "gb18030"]
    Gb18030,
    #[iden = "gb2312"]
    Gb2312,
    #[iden = "gbk"]
    Gbk,
    #[iden = "geostd8"]
    Geostd8,
    #[iden = "greek"]
    Greek,
    #[iden = "hebrew"]
    Hebrew,
    #[iden = "hp8"]
    Hp8,
    #[iden = "keybcs2"]
    Keybcs2,
    #[iden = "koi8r"]
    Koi8R,
    #[iden = "koi8u"]
    Koi8U,
    #[iden = "latin1"]
    Latin1,
    #[iden = "latin2"]
    Latin2,
    #[iden = "latin5"]
    Latin5,
    #[iden = "latin7"]
    Latin7,
    #[iden = "macce"]
    Macce,
    #[iden = "macroman"]
    Macroman,
    #[iden = "sjis"]
    Sjis,
    #[iden = "swe7"]
    Swe7,
    #[iden = "tis620"]
    Tis620,
    #[iden = "ucs2"]
    Ucs2,
    #[iden = "ujis"]
    Ujis,
    #[iden = "utf16"]
    Utf16,
    #[iden = "utf16le"]
    Utf16Le,
    #[iden = "utf32"]
    Utf32,
    #[iden = "utf8"]
    Utf8,
    #[iden = "utf8mb4"]
    Utf8Mb4,
    #[method = "unknown_to_string"]
    Unknown(String),
}

/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema-collation-character-set-applicability-table.html
#[derive(Clone, Debug, PartialEq, sea_query::Iden, sea_schema_derive::Name)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[catch = "string_to_unknown"]
pub enum Collation {
    #[iden = "armscii8_general_ci"]
    Armscii8GeneralCi,
    #[iden = "armscii8_bin"]
    Armscii8Bin,
    #[iden = "ascii_general_ci"]
    AsciiGeneralCi,
    #[iden = "ascii_bin"]
    AsciiBin,
    #[iden = "big5_chinese_ci"]
    Big5ChineseCi,
    #[iden = "big5_bin"]
    Big5Bin,
    #[iden = "binary"]
    Binary,
    #[iden = "cp1250_general_ci"]
    Cp1250GeneralCi,
    #[iden = "cp1250_czech_cs"]
    Cp1250CzechCs,
    #[iden = "cp1250_croatian_ci"]
    Cp1250CroatianCi,
    #[iden = "cp1250_bin"]
    Cp1250Bin,
    #[iden = "cp1250_polish_ci"]
    Cp1250PolishCi,
    #[iden = "cp1251_bulgarian_ci"]
    Cp1251BulgarianCi,
    #[iden = "cp1251_ukrainian_ci"]
    Cp1251UkrainianCi,
    #[iden = "cp1251_bin"]
    Cp1251Bin,
    #[iden = "cp1251_general_ci"]
    Cp1251GeneralCi,
    #[iden = "cp1251_general_cs"]
    Cp1251GeneralCs,
    #[iden = "cp1256_general_ci"]
    Cp1256GeneralCi,
    #[iden = "cp1256_bin"]
    Cp1256Bin,
    #[iden = "cp1257_lithuanian_ci"]
    Cp1257LithuanianCi,
    #[iden = "cp1257_bin"]
    Cp1257Bin,
    #[iden = "cp1257_general_ci"]
    Cp1257GeneralCi,
    #[iden = "cp850_general_ci"]
    Cp850GeneralCi,
    #[iden = "cp850_bin"]
    Cp850Bin,
    #[iden = "cp852_general_ci"]
    Cp852GeneralCi,
    #[iden = "cp852_bin"]
    Cp852Bin,
    #[iden = "cp866_general_ci"]
    Cp866GeneralCi,
    #[iden = "cp866_bin"]
    Cp866Bin,
    #[iden = "cp932_japanese_ci"]
    Cp932JapaneseCi,
    #[iden = "cp932_bin"]
    Cp932Bin,
    #[iden = "dec8_swedish_ci"]
    Dec8SwedishCi,
    #[iden = "dec8_bin"]
    Dec8Bin,
    #[iden = "eucjpms_japanese_ci"]
    EucjpmsJapaneseCi,
    #[iden = "eucjpms_bin"]
    EucjpmsBin,
    #[iden = "euckr_korean_ci"]
    EuckrKoreanCi,
    #[iden = "euckr_bin"]
    EuckrBin,
    #[iden = "gb18030_chinese_ci"]
    Gb18030ChineseCi,
    #[iden = "gb18030_bin"]
    Gb18030Bin,
    #[iden = "gb18030_unicode_520_ci"]
    Gb18030Unicode520Ci,
    #[iden = "gb2312_chinese_ci"]
    Gb2312ChineseCi,
    #[iden = "gb2312_bin"]
    Gb2312Bin,
    #[iden = "gbk_chinese_ci"]
    GbkChineseCi,
    #[iden = "gbk_bin"]
    GbkBin,
    #[iden = "geostd8_general_ci"]
    Geostd8GeneralCi,
    #[iden = "geostd8_bin"]
    Geostd8Bin,
    #[iden = "greek_general_ci"]
    GreekGeneralCi,
    #[iden = "greek_bin"]
    GreekBin,
    #[iden = "hebrew_general_ci"]
    HebrewGeneralCi,
    #[iden = "hebrew_bin"]
    HebrewBin,
    #[iden = "hp8_english_ci"]
    Hp8EnglishCi,
    #[iden = "hp8_bin"]
    Hp8Bin,
    #[iden = "keybcs2_general_ci"]
    Keybcs2GeneralCi,
    #[iden = "keybcs2_bin"]
    Keybcs2Bin,
    #[iden = "koi8r_general_ci"]
    Koi8RGeneralCi,
    #[iden = "koi8r_bin"]
    Koi8RBin,
    #[iden = "koi8u_general_ci"]
    Koi8UGeneralCi,
    #[iden = "koi8u_bin"]
    Koi8UBin,
    #[iden = "latin1_german1_ci"]
    Latin1German1Ci,
    #[iden = "latin1_swedish_ci"]
    Latin1SwedishCi,
    #[iden = "latin1_danish_ci"]
    Latin1DanishCi,
    #[iden = "latin1_german2_ci"]
    Latin1German2Ci,
    #[iden = "latin1_bin"]
    Latin1Bin,
    #[iden = "latin1_general_ci"]
    Latin1GeneralCi,
    #[iden = "latin1_general_cs"]
    Latin1GeneralCs,
    #[iden = "latin1_spanish_ci"]
    Latin1SpanishCi,
    #[iden = "latin2_czech_cs"]
    Latin2CzechCs,
    #[iden = "latin2_general_ci"]
    Latin2GeneralCi,
    #[iden = "latin2_hungarian_ci"]
    Latin2HungarianCi,
    #[iden = "latin2_croatian_ci"]
    Latin2CroatianCi,
    #[iden = "latin2_bin"]
    Latin2Bin,
    #[iden = "latin5_turkish_ci"]
    Latin5TurkishCi,
    #[iden = "latin5_bin"]
    Latin5Bin,
    #[iden = "latin7_estonian_cs"]
    Latin7EstonianCs,
    #[iden = "latin7_general_ci"]
    Latin7GeneralCi,
    #[iden = "latin7_general_cs"]
    Latin7GeneralCs,
    #[iden = "latin7_bin"]
    Latin7Bin,
    #[iden = "macce_general_ci"]
    MacceGeneralCi,
    #[iden = "macce_bin"]
    MacceBin,
    #[iden = "macroman_general_ci"]
    MacromanGeneralCi,
    #[iden = "macroman_bin"]
    MacromanBin,
    #[iden = "sjis_japanese_ci"]
    SjisJapaneseCi,
    #[iden = "sjis_bin"]
    SjisBin,
    #[iden = "swe7_swedish_ci"]
    Swe7SwedishCi,
    #[iden = "swe7_bin"]
    Swe7Bin,
    #[iden = "tis620_thai_ci"]
    Tis620ThaiCi,
    #[iden = "tis620_bin"]
    Tis620Bin,
    #[iden = "ucs2_general_ci"]
    Ucs2GeneralCi,
    #[iden = "ucs2_bin"]
    Ucs2Bin,
    #[iden = "ucs2_unicode_ci"]
    Ucs2UnicodeCi,
    #[iden = "ucs2_icelandic_ci"]
    Ucs2IcelandicCi,
    #[iden = "ucs2_latvian_ci"]
    Ucs2LatvianCi,
    #[iden = "ucs2_romanian_ci"]
    Ucs2RomanianCi,
    #[iden = "ucs2_slovenian_ci"]
    Ucs2SlovenianCi,
    #[iden = "ucs2_polish_ci"]
    Ucs2PolishCi,
    #[iden = "ucs2_estonian_ci"]
    Ucs2EstonianCi,
    #[iden = "ucs2_spanish_ci"]
    Ucs2SpanishCi,
    #[iden = "ucs2_swedish_ci"]
    Ucs2SwedishCi,
    #[iden = "ucs2_turkish_ci"]
    Ucs2TurkishCi,
    #[iden = "ucs2_czech_ci"]
    Ucs2CzechCi,
    #[iden = "ucs2_danish_ci"]
    Ucs2DanishCi,
    #[iden = "ucs2_lithuanian_ci"]
    Ucs2LithuanianCi,
    #[iden = "ucs2_slovak_ci"]
    Ucs2SlovakCi,
    #[iden = "ucs2_spanish2_ci"]
    Ucs2Spanish2Ci,
    #[iden = "ucs2_roman_ci"]
    Ucs2RomanCi,
    #[iden = "ucs2_persian_ci"]
    Ucs2PersianCi,
    #[iden = "ucs2_esperanto_ci"]
    Ucs2EsperantoCi,
    #[iden = "ucs2_hungarian_ci"]
    Ucs2HungarianCi,
    #[iden = "ucs2_sinhala_ci"]
    Ucs2SinhalaCi,
    #[iden = "ucs2_german2_ci"]
    Ucs2German2Ci,
    #[iden = "ucs2_croatian_ci"]
    Ucs2CroatianCi,
    #[iden = "ucs2_unicode_520_ci"]
    Ucs2Unicode520Ci,
    #[iden = "ucs2_vietnamese_ci"]
    Ucs2VietnameseCi,
    #[iden = "ucs2_general_mysql500_ci"]
    Ucs2GeneralMysql500Ci,
    #[iden = "ujis_japanese_ci"]
    UjisJapaneseCi,
    #[iden = "ujis_bin"]
    UjisBin,
    #[iden = "utf16_general_ci"]
    Utf16GeneralCi,
    #[iden = "utf16_bin"]
    Utf16Bin,
    #[iden = "utf16_unicode_ci"]
    Utf16UnicodeCi,
    #[iden = "utf16_icelandic_ci"]
    Utf16IcelandicCi,
    #[iden = "utf16_latvian_ci"]
    Utf16LatvianCi,
    #[iden = "utf16_romanian_ci"]
    Utf16RomanianCi,
    #[iden = "utf16_slovenian_ci"]
    Utf16SlovenianCi,
    #[iden = "utf16_polish_ci"]
    Utf16PolishCi,
    #[iden = "utf16_estonian_ci"]
    Utf16EstonianCi,
    #[iden = "utf16_spanish_ci"]
    Utf16SpanishCi,
    #[iden = "utf16_swedish_ci"]
    Utf16SwedishCi,
    #[iden = "utf16_turkish_ci"]
    Utf16TurkishCi,
    #[iden = "utf16_czech_ci"]
    Utf16CzechCi,
    #[iden = "utf16_danish_ci"]
    Utf16DanishCi,
    #[iden = "utf16_lithuanian_ci"]
    Utf16LithuanianCi,
    #[iden = "utf16_slovak_ci"]
    Utf16SlovakCi,
    #[iden = "utf16_spanish2_ci"]
    Utf16Spanish2Ci,
    #[iden = "utf16_roman_ci"]
    Utf16RomanCi,
    #[iden = "utf16_persian_ci"]
    Utf16PersianCi,
    #[iden = "utf16_esperanto_ci"]
    Utf16EsperantoCi,
    #[iden = "utf16_hungarian_ci"]
    Utf16HungarianCi,
    #[iden = "utf16_sinhala_ci"]
    Utf16SinhalaCi,
    #[iden = "utf16_german2_ci"]
    Utf16German2Ci,
    #[iden = "utf16_croatian_ci"]
    Utf16CroatianCi,
    #[iden = "utf16_unicode_520_ci"]
    Utf16Unicode520Ci,
    #[iden = "utf16_vietnamese_ci"]
    Utf16VietnameseCi,
    #[iden = "utf16le_general_ci"]
    Utf16LeGeneralCi,
    #[iden = "utf16le_bin"]
    Utf16LeBin,
    #[iden = "utf32_general_ci"]
    Utf32GeneralCi,
    #[iden = "utf32_bin"]
    Utf32Bin,
    #[iden = "utf32_unicode_ci"]
    Utf32UnicodeCi,
    #[iden = "utf32_icelandic_ci"]
    Utf32IcelandicCi,
    #[iden = "utf32_latvian_ci"]
    Utf32LatvianCi,
    #[iden = "utf32_romanian_ci"]
    Utf32RomanianCi,
    #[iden = "utf32_slovenian_ci"]
    Utf32SlovenianCi,
    #[iden = "utf32_polish_ci"]
    Utf32PolishCi,
    #[iden = "utf32_estonian_ci"]
    Utf32EstonianCi,
    #[iden = "utf32_spanish_ci"]
    Utf32SpanishCi,
    #[iden = "utf32_swedish_ci"]
    Utf32SwedishCi,
    #[iden = "utf32_turkish_ci"]
    Utf32TurkishCi,
    #[iden = "utf32_czech_ci"]
    Utf32CzechCi,
    #[iden = "utf32_danish_ci"]
    Utf32DanishCi,
    #[iden = "utf32_lithuanian_ci"]
    Utf32LithuanianCi,
    #[iden = "utf32_slovak_ci"]
    Utf32SlovakCi,
    #[iden = "utf32_spanish2_ci"]
    Utf32Spanish2Ci,
    #[iden = "utf32_roman_ci"]
    Utf32RomanCi,
    #[iden = "utf32_persian_ci"]
    Utf32PersianCi,
    #[iden = "utf32_esperanto_ci"]
    Utf32EsperantoCi,
    #[iden = "utf32_hungarian_ci"]
    Utf32HungarianCi,
    #[iden = "utf32_sinhala_ci"]
    Utf32SinhalaCi,
    #[iden = "utf32_german2_ci"]
    Utf32German2Ci,
    #[iden = "utf32_croatian_ci"]
    Utf32CroatianCi,
    #[iden = "utf32_unicode_520_ci"]
    Utf32Unicode520Ci,
    #[iden = "utf32_vietnamese_ci"]
    Utf32VietnameseCi,
    #[iden = "utf8_general_ci"]
    Utf8GeneralCi,
    #[iden = "utf8_tolower_ci"]
    Utf8TolowerCi,
    #[iden = "utf8_bin"]
    Utf8Bin,
    #[iden = "utf8_unicode_ci"]
    Utf8UnicodeCi,
    #[iden = "utf8_icelandic_ci"]
    Utf8IcelandicCi,
    #[iden = "utf8_latvian_ci"]
    Utf8LatvianCi,
    #[iden = "utf8_romanian_ci"]
    Utf8RomanianCi,
    #[iden = "utf8_slovenian_ci"]
    Utf8SlovenianCi,
    #[iden = "utf8_polish_ci"]
    Utf8PolishCi,
    #[iden = "utf8_estonian_ci"]
    Utf8EstonianCi,
    #[iden = "utf8_spanish_ci"]
    Utf8SpanishCi,
    #[iden = "utf8_swedish_ci"]
    Utf8SwedishCi,
    #[iden = "utf8_turkish_ci"]
    Utf8TurkishCi,
    #[iden = "utf8_czech_ci"]
    Utf8CzechCi,
    #[iden = "utf8_danish_ci"]
    Utf8DanishCi,
    #[iden = "utf8_lithuanian_ci"]
    Utf8LithuanianCi,
    #[iden = "utf8_slovak_ci"]
    Utf8SlovakCi,
    #[iden = "utf8_spanish2_ci"]
    Utf8Spanish2Ci,
    #[iden = "utf8_roman_ci"]
    Utf8RomanCi,
    #[iden = "utf8_persian_ci"]
    Utf8PersianCi,
    #[iden = "utf8_esperanto_ci"]
    Utf8EsperantoCi,
    #[iden = "utf8_hungarian_ci"]
    Utf8HungarianCi,
    #[iden = "utf8_sinhala_ci"]
    Utf8SinhalaCi,
    #[iden = "utf8_german2_ci"]
    Utf8German2Ci,
    #[iden = "utf8_croatian_ci"]
    Utf8CroatianCi,
    #[iden = "utf8_unicode_520_ci"]
    Utf8Unicode520Ci,
    #[iden = "utf8_vietnamese_ci"]
    Utf8VietnameseCi,
    #[iden = "utf8_general_mysql500_ci"]
    Utf8GeneralMysql500Ci,
    #[iden = "utf8mb4_general_ci"]
    Utf8Mb4GeneralCi,
    #[iden = "utf8mb4_bin"]
    Utf8Mb4Bin,
    #[iden = "utf8mb4_unicode_ci"]
    Utf8Mb4UnicodeCi,
    #[iden = "utf8mb4_icelandic_ci"]
    Utf8Mb4IcelandicCi,
    #[iden = "utf8mb4_latvian_ci"]
    Utf8Mb4LatvianCi,
    #[iden = "utf8mb4_romanian_ci"]
    Utf8Mb4RomanianCi,
    #[iden = "utf8mb4_slovenian_ci"]
    Utf8Mb4SlovenianCi,
    #[iden = "utf8mb4_polish_ci"]
    Utf8Mb4PolishCi,
    #[iden = "utf8mb4_estonian_ci"]
    Utf8Mb4EstonianCi,
    #[iden = "utf8mb4_spanish_ci"]
    Utf8Mb4SpanishCi,
    #[iden = "utf8mb4_swedish_ci"]
    Utf8Mb4SwedishCi,
    #[iden = "utf8mb4_turkish_ci"]
    Utf8Mb4TurkishCi,
    #[iden = "utf8mb4_czech_ci"]
    Utf8Mb4CzechCi,
    #[iden = "utf8mb4_danish_ci"]
    Utf8Mb4DanishCi,
    #[iden = "utf8mb4_lithuanian_ci"]
    Utf8Mb4LithuanianCi,
    #[iden = "utf8mb4_slovak_ci"]
    Utf8Mb4SlovakCi,
    #[iden = "utf8mb4_spanish2_ci"]
    Utf8Mb4Spanish2Ci,
    #[iden = "utf8mb4_roman_ci"]
    Utf8Mb4RomanCi,
    #[iden = "utf8mb4_persian_ci"]
    Utf8Mb4PersianCi,
    #[iden = "utf8mb4_esperanto_ci"]
    Utf8Mb4EsperantoCi,
    #[iden = "utf8mb4_hungarian_ci"]
    Utf8Mb4HungarianCi,
    #[iden = "utf8mb4_sinhala_ci"]
    Utf8Mb4SinhalaCi,
    #[iden = "utf8mb4_german2_ci"]
    Utf8Mb4German2Ci,
    #[iden = "utf8mb4_croatian_ci"]
    Utf8Mb4CroatianCi,
    #[iden = "utf8mb4_unicode_520_ci"]
    Utf8Mb4Unicode520Ci,
    #[iden = "utf8mb4_vietnamese_ci"]
    Utf8Mb4VietnameseCi,
    #[iden = "utf8mb4_0900_ai_ci"]
    Utf8Mb40900AiCi,
    #[iden = "utf8mb4_de_pb_0900_ai_ci"]
    Utf8Mb4DePb0900AiCi,
    #[iden = "utf8mb4_is_0900_ai_ci"]
    Utf8Mb4Is0900AiCi,
    #[iden = "utf8mb4_lv_0900_ai_ci"]
    Utf8Mb4Lv0900AiCi,
    #[iden = "utf8mb4_ro_0900_ai_ci"]
    Utf8Mb4Ro0900AiCi,
    #[iden = "utf8mb4_sl_0900_ai_ci"]
    Utf8Mb4Sl0900AiCi,
    #[iden = "utf8mb4_pl_0900_ai_ci"]
    Utf8Mb4Pl0900AiCi,
    #[iden = "utf8mb4_et_0900_ai_ci"]
    Utf8Mb4Et0900AiCi,
    #[iden = "utf8mb4_es_0900_ai_ci"]
    Utf8Mb4Es0900AiCi,
    #[iden = "utf8mb4_sv_0900_ai_ci"]
    Utf8Mb4Sv0900AiCi,
    #[iden = "utf8mb4_tr_0900_ai_ci"]
    Utf8Mb4Tr0900AiCi,
    #[iden = "utf8mb4_cs_0900_ai_ci"]
    Utf8Mb4Cs0900AiCi,
    #[iden = "utf8mb4_da_0900_ai_ci"]
    Utf8Mb4Da0900AiCi,
    #[iden = "utf8mb4_lt_0900_ai_ci"]
    Utf8Mb4Lt0900AiCi,
    #[iden = "utf8mb4_sk_0900_ai_ci"]
    Utf8Mb4Sk0900AiCi,
    #[iden = "utf8mb4_es_trad_0900_ai_ci"]
    Utf8Mb4EsTrad0900AiCi,
    #[iden = "utf8mb4_la_0900_ai_ci"]
    Utf8Mb4La0900AiCi,
    #[iden = "utf8mb4_eo_0900_ai_ci"]
    Utf8Mb4Eo0900AiCi,
    #[iden = "utf8mb4_hu_0900_ai_ci"]
    Utf8Mb4Hu0900AiCi,
    #[iden = "utf8mb4_hr_0900_ai_ci"]
    Utf8Mb4Hr0900AiCi,
    #[iden = "utf8mb4_vi_0900_ai_ci"]
    Utf8Mb4Vi0900AiCi,
    #[iden = "utf8mb4_0900_as_cs"]
    Utf8Mb40900AsCs,
    #[iden = "utf8mb4_de_pb_0900_as_cs"]
    Utf8Mb4DePb0900AsCs,
    #[iden = "utf8mb4_is_0900_as_cs"]
    Utf8Mb4Is0900AsCs,
    #[iden = "utf8mb4_lv_0900_as_cs"]
    Utf8Mb4Lv0900AsCs,
    #[iden = "utf8mb4_ro_0900_as_cs"]
    Utf8Mb4Ro0900AsCs,
    #[iden = "utf8mb4_sl_0900_as_cs"]
    Utf8Mb4Sl0900AsCs,
    #[iden = "utf8mb4_pl_0900_as_cs"]
    Utf8Mb4Pl0900AsCs,
    #[iden = "utf8mb4_et_0900_as_cs"]
    Utf8Mb4Et0900AsCs,
    #[iden = "utf8mb4_es_0900_as_cs"]
    Utf8Mb4Es0900AsCs,
    #[iden = "utf8mb4_sv_0900_as_cs"]
    Utf8Mb4Sv0900AsCs,
    #[iden = "utf8mb4_tr_0900_as_cs"]
    Utf8Mb4Tr0900AsCs,
    #[iden = "utf8mb4_cs_0900_as_cs"]
    Utf8Mb4Cs0900AsCs,
    #[iden = "utf8mb4_da_0900_as_cs"]
    Utf8Mb4Da0900AsCs,
    #[iden = "utf8mb4_lt_0900_as_cs"]
    Utf8Mb4Lt0900AsCs,
    #[iden = "utf8mb4_sk_0900_as_cs"]
    Utf8Mb4Sk0900AsCs,
    #[iden = "utf8mb4_es_trad_0900_as_cs"]
    Utf8Mb4EsTrad0900AsCs,
    #[iden = "utf8mb4_la_0900_as_cs"]
    Utf8Mb4La0900AsCs,
    #[iden = "utf8mb4_eo_0900_as_cs"]
    Utf8Mb4Eo0900AsCs,
    #[iden = "utf8mb4_hu_0900_as_cs"]
    Utf8Mb4Hu0900AsCs,
    #[iden = "utf8mb4_hr_0900_as_cs"]
    Utf8Mb4Hr0900AsCs,
    #[iden = "utf8mb4_vi_0900_as_cs"]
    Utf8Mb4Vi0900AsCs,
    #[iden = "utf8mb4_ja_0900_as_cs"]
    Utf8Mb4Ja0900AsCs,
    #[iden = "utf8mb4_ja_0900_as_cs_ks"]
    Utf8Mb4Ja0900AsCsKs,
    #[iden = "utf8mb4_0900_as_ci"]
    Utf8Mb40900AsCi,
    #[iden = "utf8mb4_ru_0900_ai_ci"]
    Utf8Mb4Ru0900AiCi,
    #[iden = "utf8mb4_ru_0900_as_cs"]
    Utf8Mb4Ru0900AsCs,
    #[iden = "utf8mb4_zh_0900_as_cs"]
    Utf8Mb4Zh0900AsCs,
    #[iden = "utf8mb4_0900_bin"]
    Utf8Mb40900Bin,
    #[method = "unknown_to_string"]
    Unknown(String),
}

impl CharSet {
    pub fn description(&self) -> String {
        match self {
            Self::Armscii8 => "ARMSCII-8 Armenian".to_owned(),
            Self::Ascii => "US ASCII".to_owned(),
            Self::Big5 => "Big5 Traditional Chinese".to_owned(),
            Self::Binary => "Binary pseudo charset".to_owned(),
            Self::Cp1250 => "Windows Central European".to_owned(),
            Self::Cp1251 => "Windows Cyrillic".to_owned(),
            Self::Cp1256 => "Windows Arabic".to_owned(),
            Self::Cp1257 => "Windows Baltic".to_owned(),
            Self::Cp850 => "DOS West European".to_owned(),
            Self::Cp852 => "DOS Central European".to_owned(),
            Self::Cp866 => "DOS Russian".to_owned(),
            Self::Cp932 => "SJIS for Windows Japanese".to_owned(),
            Self::Dec8 => "DEC West European".to_owned(),
            Self::Eucjpms => "UJIS for Windows Japanese".to_owned(),
            Self::Euckr => "EUC-KR Korean".to_owned(),
            Self::Gb18030 => "China National Standard GB18030".to_owned(),
            Self::Gb2312 => "GB2312 Simplified Chinese".to_owned(),
            Self::Gbk => "GBK Simplified Chinese".to_owned(),
            Self::Geostd8 => "GEOSTD8 Georgian".to_owned(),
            Self::Greek => "ISO 8859-7 Greek".to_owned(),
            Self::Hebrew => "ISO 8859-8 Hebrew".to_owned(),
            Self::Hp8 => "HP West European".to_owned(),
            Self::Keybcs2 => "DOS Kamenicky Czech-Slovak".to_owned(),
            Self::Koi8R => "KOI8-R Relcom Russian".to_owned(),
            Self::Koi8U => "KOI8-U Ukrainian".to_owned(),
            Self::Latin1 => "cp1252 West European".to_owned(),
            Self::Latin2 => "ISO 8859-2 Central European".to_owned(),
            Self::Latin5 => "ISO 8859-9 Turkish".to_owned(),
            Self::Latin7 => "ISO 8859-13 Baltic".to_owned(),
            Self::Macce => "Mac Central European".to_owned(),
            Self::Macroman => "Mac West European".to_owned(),
            Self::Sjis => "Shift-JIS Japanese".to_owned(),
            Self::Swe7 => "7bit Swedish".to_owned(),
            Self::Tis620 => "TIS620 Thai".to_owned(),
            Self::Ucs2 => "UCS-2 Unicode".to_owned(),
            Self::Ujis => "EUC-JP Japanese".to_owned(),
            Self::Utf16 => "UTF-16 Unicode".to_owned(),
            Self::Utf16Le => "UTF-16LE Unicode".to_owned(),
            Self::Utf32 => "UTF-32 Unicode".to_owned(),
            Self::Utf8 => "UTF-8 Unicode".to_owned(),
            Self::Utf8Mb4 => "UTF-8 Unicode".to_owned(),
            Self::Unknown(_) => "Unknown".to_owned(),
        }
    }

    pub fn default_collation(&self) -> Collation {
        match self {
            Self::Armscii8 => Collation::Armscii8GeneralCi,
            Self::Ascii => Collation::AsciiGeneralCi,
            Self::Big5 => Collation::Big5ChineseCi,
            Self::Binary => Collation::Binary,
            Self::Cp1250 => Collation::Cp1250GeneralCi,
            Self::Cp1251 => Collation::Cp1251GeneralCi,
            Self::Cp1256 => Collation::Cp1256GeneralCi,
            Self::Cp1257 => Collation::Cp1257GeneralCi,
            Self::Cp850 => Collation::Cp850GeneralCi,
            Self::Cp852 => Collation::Cp852GeneralCi,
            Self::Cp866 => Collation::Cp866GeneralCi,
            Self::Cp932 => Collation::Cp932JapaneseCi,
            Self::Dec8 => Collation::Dec8SwedishCi,
            Self::Eucjpms => Collation::EucjpmsJapaneseCi,
            Self::Euckr => Collation::EuckrKoreanCi,
            Self::Gb18030 => Collation::Gb18030ChineseCi,
            Self::Gb2312 => Collation::Gb2312ChineseCi,
            Self::Gbk => Collation::GbkChineseCi,
            Self::Geostd8 => Collation::Geostd8GeneralCi,
            Self::Greek => Collation::GreekGeneralCi,
            Self::Hebrew => Collation::HebrewGeneralCi,
            Self::Hp8 => Collation::Hp8EnglishCi,
            Self::Keybcs2 => Collation::Keybcs2GeneralCi,
            Self::Koi8R => Collation::Koi8RGeneralCi,
            Self::Koi8U => Collation::Koi8UGeneralCi,
            Self::Latin1 => Collation::Latin1SwedishCi,
            Self::Latin2 => Collation::Latin2GeneralCi,
            Self::Latin5 => Collation::Latin5TurkishCi,
            Self::Latin7 => Collation::Latin7GeneralCi,
            Self::Macce => Collation::MacceGeneralCi,
            Self::Macroman => Collation::MacromanGeneralCi,
            Self::Sjis => Collation::SjisJapaneseCi,
            Self::Swe7 => Collation::Swe7SwedishCi,
            Self::Tis620 => Collation::Tis620ThaiCi,
            Self::Ucs2 => Collation::Ucs2GeneralCi,
            Self::Ujis => Collation::UjisJapaneseCi,
            Self::Utf16 => Collation::Utf16GeneralCi,
            Self::Utf16Le => Collation::Utf16LeGeneralCi,
            Self::Utf32 => Collation::Utf32GeneralCi,
            Self::Utf8 => Collation::Utf8GeneralCi,
            Self::Utf8Mb4 => Collation::Utf8Mb40900AiCi,
            Self::Unknown(_) => panic!("unknown"),
        }
    }

    pub fn max_len(&self) -> u32 {
        match self {
            Self::Armscii8 => 1,
            Self::Ascii => 1,
            Self::Big5 => 2,
            Self::Binary => 1,
            Self::Cp1250 => 1,
            Self::Cp1251 => 1,
            Self::Cp1256 => 1,
            Self::Cp1257 => 1,
            Self::Cp850 => 1,
            Self::Cp852 => 1,
            Self::Cp866 => 1,
            Self::Cp932 => 2,
            Self::Dec8 => 1,
            Self::Eucjpms => 3,
            Self::Euckr => 2,
            Self::Gb18030 => 4,
            Self::Gb2312 => 2,
            Self::Gbk => 2,
            Self::Geostd8 => 1,
            Self::Greek => 1,
            Self::Hebrew => 1,
            Self::Hp8 => 1,
            Self::Keybcs2 => 1,
            Self::Koi8R => 1,
            Self::Koi8U => 1,
            Self::Latin1 => 1,
            Self::Latin2 => 1,
            Self::Latin5 => 1,
            Self::Latin7 => 1,
            Self::Macce => 1,
            Self::Macroman => 1,
            Self::Sjis => 2,
            Self::Swe7 => 1,
            Self::Tis620 => 1,
            Self::Ucs2 => 2,
            Self::Ujis => 3,
            Self::Utf16 => 4,
            Self::Utf16Le => 4,
            Self::Utf32 => 4,
            Self::Utf8 => 3,
            Self::Utf8Mb4 => 4,
            Self::Unknown(_) => panic!("unknown"),
        }
    }

    pub fn unknown_to_string(&self) -> &String {
        match self {
            Self::Unknown(custom) => custom,
            _ => panic!("not Unknown"),
        }
    }

    pub fn string_to_unknown(string: &str) -> Option<Self> {
        Some(Self::Unknown(string.to_string()))
    }
}

impl Collation {
    pub fn char_set(&self) -> CharSet {
        match self {
            Self::Armscii8GeneralCi => CharSet::Armscii8,
            Self::Armscii8Bin => CharSet::Armscii8,
            Self::AsciiGeneralCi => CharSet::Ascii,
            Self::AsciiBin => CharSet::Ascii,
            Self::Big5ChineseCi => CharSet::Big5,
            Self::Big5Bin => CharSet::Big5,
            Self::Binary => CharSet::Binary,
            Self::Cp1250GeneralCi => CharSet::Cp1250,
            Self::Cp1250CzechCs => CharSet::Cp1250,
            Self::Cp1250CroatianCi => CharSet::Cp1250,
            Self::Cp1250Bin => CharSet::Cp1250,
            Self::Cp1250PolishCi => CharSet::Cp1250,
            Self::Cp1251BulgarianCi => CharSet::Cp1251,
            Self::Cp1251UkrainianCi => CharSet::Cp1251,
            Self::Cp1251Bin => CharSet::Cp1251,
            Self::Cp1251GeneralCi => CharSet::Cp1251,
            Self::Cp1251GeneralCs => CharSet::Cp1251,
            Self::Cp1256GeneralCi => CharSet::Cp1256,
            Self::Cp1256Bin => CharSet::Cp1256,
            Self::Cp1257LithuanianCi => CharSet::Cp1257,
            Self::Cp1257Bin => CharSet::Cp1257,
            Self::Cp1257GeneralCi => CharSet::Cp1257,
            Self::Cp850GeneralCi => CharSet::Cp850,
            Self::Cp850Bin => CharSet::Cp850,
            Self::Cp852GeneralCi => CharSet::Cp852,
            Self::Cp852Bin => CharSet::Cp852,
            Self::Cp866GeneralCi => CharSet::Cp866,
            Self::Cp866Bin => CharSet::Cp866,
            Self::Cp932JapaneseCi => CharSet::Cp932,
            Self::Cp932Bin => CharSet::Cp932,
            Self::Dec8SwedishCi => CharSet::Dec8,
            Self::Dec8Bin => CharSet::Dec8,
            Self::EucjpmsJapaneseCi => CharSet::Eucjpms,
            Self::EucjpmsBin => CharSet::Eucjpms,
            Self::EuckrKoreanCi => CharSet::Euckr,
            Self::EuckrBin => CharSet::Euckr,
            Self::Gb18030ChineseCi => CharSet::Gb18030,
            Self::Gb18030Bin => CharSet::Gb18030,
            Self::Gb18030Unicode520Ci => CharSet::Gb18030,
            Self::Gb2312ChineseCi => CharSet::Gb2312,
            Self::Gb2312Bin => CharSet::Gb2312,
            Self::GbkChineseCi => CharSet::Gbk,
            Self::GbkBin => CharSet::Gbk,
            Self::Geostd8GeneralCi => CharSet::Geostd8,
            Self::Geostd8Bin => CharSet::Geostd8,
            Self::GreekGeneralCi => CharSet::Greek,
            Self::GreekBin => CharSet::Greek,
            Self::HebrewGeneralCi => CharSet::Hebrew,
            Self::HebrewBin => CharSet::Hebrew,
            Self::Hp8EnglishCi => CharSet::Hp8,
            Self::Hp8Bin => CharSet::Hp8,
            Self::Keybcs2GeneralCi => CharSet::Keybcs2,
            Self::Keybcs2Bin => CharSet::Keybcs2,
            Self::Koi8RGeneralCi => CharSet::Koi8R,
            Self::Koi8RBin => CharSet::Koi8R,
            Self::Koi8UGeneralCi => CharSet::Koi8U,
            Self::Koi8UBin => CharSet::Koi8U,
            Self::Latin1German1Ci => CharSet::Latin1,
            Self::Latin1SwedishCi => CharSet::Latin1,
            Self::Latin1DanishCi => CharSet::Latin1,
            Self::Latin1German2Ci => CharSet::Latin1,
            Self::Latin1Bin => CharSet::Latin1,
            Self::Latin1GeneralCi => CharSet::Latin1,
            Self::Latin1GeneralCs => CharSet::Latin1,
            Self::Latin1SpanishCi => CharSet::Latin1,
            Self::Latin2CzechCs => CharSet::Latin2,
            Self::Latin2GeneralCi => CharSet::Latin2,
            Self::Latin2HungarianCi => CharSet::Latin2,
            Self::Latin2CroatianCi => CharSet::Latin2,
            Self::Latin2Bin => CharSet::Latin2,
            Self::Latin5TurkishCi => CharSet::Latin5,
            Self::Latin5Bin => CharSet::Latin5,
            Self::Latin7EstonianCs => CharSet::Latin7,
            Self::Latin7GeneralCi => CharSet::Latin7,
            Self::Latin7GeneralCs => CharSet::Latin7,
            Self::Latin7Bin => CharSet::Latin7,
            Self::MacceGeneralCi => CharSet::Macce,
            Self::MacceBin => CharSet::Macce,
            Self::MacromanGeneralCi => CharSet::Macroman,
            Self::MacromanBin => CharSet::Macroman,
            Self::SjisJapaneseCi => CharSet::Sjis,
            Self::SjisBin => CharSet::Sjis,
            Self::Swe7SwedishCi => CharSet::Swe7,
            Self::Swe7Bin => CharSet::Swe7,
            Self::Tis620ThaiCi => CharSet::Tis620,
            Self::Tis620Bin => CharSet::Tis620,
            Self::Ucs2GeneralCi => CharSet::Ucs2,
            Self::Ucs2Bin => CharSet::Ucs2,
            Self::Ucs2UnicodeCi => CharSet::Ucs2,
            Self::Ucs2IcelandicCi => CharSet::Ucs2,
            Self::Ucs2LatvianCi => CharSet::Ucs2,
            Self::Ucs2RomanianCi => CharSet::Ucs2,
            Self::Ucs2SlovenianCi => CharSet::Ucs2,
            Self::Ucs2PolishCi => CharSet::Ucs2,
            Self::Ucs2EstonianCi => CharSet::Ucs2,
            Self::Ucs2SpanishCi => CharSet::Ucs2,
            Self::Ucs2SwedishCi => CharSet::Ucs2,
            Self::Ucs2TurkishCi => CharSet::Ucs2,
            Self::Ucs2CzechCi => CharSet::Ucs2,
            Self::Ucs2DanishCi => CharSet::Ucs2,
            Self::Ucs2LithuanianCi => CharSet::Ucs2,
            Self::Ucs2SlovakCi => CharSet::Ucs2,
            Self::Ucs2Spanish2Ci => CharSet::Ucs2,
            Self::Ucs2RomanCi => CharSet::Ucs2,
            Self::Ucs2PersianCi => CharSet::Ucs2,
            Self::Ucs2EsperantoCi => CharSet::Ucs2,
            Self::Ucs2HungarianCi => CharSet::Ucs2,
            Self::Ucs2SinhalaCi => CharSet::Ucs2,
            Self::Ucs2German2Ci => CharSet::Ucs2,
            Self::Ucs2CroatianCi => CharSet::Ucs2,
            Self::Ucs2Unicode520Ci => CharSet::Ucs2,
            Self::Ucs2VietnameseCi => CharSet::Ucs2,
            Self::Ucs2GeneralMysql500Ci => CharSet::Ucs2,
            Self::UjisJapaneseCi => CharSet::Ujis,
            Self::UjisBin => CharSet::Ujis,
            Self::Utf16GeneralCi => CharSet::Utf16,
            Self::Utf16Bin => CharSet::Utf16,
            Self::Utf16UnicodeCi => CharSet::Utf16,
            Self::Utf16IcelandicCi => CharSet::Utf16,
            Self::Utf16LatvianCi => CharSet::Utf16,
            Self::Utf16RomanianCi => CharSet::Utf16,
            Self::Utf16SlovenianCi => CharSet::Utf16,
            Self::Utf16PolishCi => CharSet::Utf16,
            Self::Utf16EstonianCi => CharSet::Utf16,
            Self::Utf16SpanishCi => CharSet::Utf16,
            Self::Utf16SwedishCi => CharSet::Utf16,
            Self::Utf16TurkishCi => CharSet::Utf16,
            Self::Utf16CzechCi => CharSet::Utf16,
            Self::Utf16DanishCi => CharSet::Utf16,
            Self::Utf16LithuanianCi => CharSet::Utf16,
            Self::Utf16SlovakCi => CharSet::Utf16,
            Self::Utf16Spanish2Ci => CharSet::Utf16,
            Self::Utf16RomanCi => CharSet::Utf16,
            Self::Utf16PersianCi => CharSet::Utf16,
            Self::Utf16EsperantoCi => CharSet::Utf16,
            Self::Utf16HungarianCi => CharSet::Utf16,
            Self::Utf16SinhalaCi => CharSet::Utf16,
            Self::Utf16German2Ci => CharSet::Utf16,
            Self::Utf16CroatianCi => CharSet::Utf16,
            Self::Utf16Unicode520Ci => CharSet::Utf16,
            Self::Utf16VietnameseCi => CharSet::Utf16,
            Self::Utf16LeGeneralCi => CharSet::Utf16Le,
            Self::Utf16LeBin => CharSet::Utf16Le,
            Self::Utf32GeneralCi => CharSet::Utf32,
            Self::Utf32Bin => CharSet::Utf32,
            Self::Utf32UnicodeCi => CharSet::Utf32,
            Self::Utf32IcelandicCi => CharSet::Utf32,
            Self::Utf32LatvianCi => CharSet::Utf32,
            Self::Utf32RomanianCi => CharSet::Utf32,
            Self::Utf32SlovenianCi => CharSet::Utf32,
            Self::Utf32PolishCi => CharSet::Utf32,
            Self::Utf32EstonianCi => CharSet::Utf32,
            Self::Utf32SpanishCi => CharSet::Utf32,
            Self::Utf32SwedishCi => CharSet::Utf32,
            Self::Utf32TurkishCi => CharSet::Utf32,
            Self::Utf32CzechCi => CharSet::Utf32,
            Self::Utf32DanishCi => CharSet::Utf32,
            Self::Utf32LithuanianCi => CharSet::Utf32,
            Self::Utf32SlovakCi => CharSet::Utf32,
            Self::Utf32Spanish2Ci => CharSet::Utf32,
            Self::Utf32RomanCi => CharSet::Utf32,
            Self::Utf32PersianCi => CharSet::Utf32,
            Self::Utf32EsperantoCi => CharSet::Utf32,
            Self::Utf32HungarianCi => CharSet::Utf32,
            Self::Utf32SinhalaCi => CharSet::Utf32,
            Self::Utf32German2Ci => CharSet::Utf32,
            Self::Utf32CroatianCi => CharSet::Utf32,
            Self::Utf32Unicode520Ci => CharSet::Utf32,
            Self::Utf32VietnameseCi => CharSet::Utf32,
            Self::Utf8GeneralCi => CharSet::Utf8,
            Self::Utf8TolowerCi => CharSet::Utf8,
            Self::Utf8Bin => CharSet::Utf8,
            Self::Utf8UnicodeCi => CharSet::Utf8,
            Self::Utf8IcelandicCi => CharSet::Utf8,
            Self::Utf8LatvianCi => CharSet::Utf8,
            Self::Utf8RomanianCi => CharSet::Utf8,
            Self::Utf8SlovenianCi => CharSet::Utf8,
            Self::Utf8PolishCi => CharSet::Utf8,
            Self::Utf8EstonianCi => CharSet::Utf8,
            Self::Utf8SpanishCi => CharSet::Utf8,
            Self::Utf8SwedishCi => CharSet::Utf8,
            Self::Utf8TurkishCi => CharSet::Utf8,
            Self::Utf8CzechCi => CharSet::Utf8,
            Self::Utf8DanishCi => CharSet::Utf8,
            Self::Utf8LithuanianCi => CharSet::Utf8,
            Self::Utf8SlovakCi => CharSet::Utf8,
            Self::Utf8Spanish2Ci => CharSet::Utf8,
            Self::Utf8RomanCi => CharSet::Utf8,
            Self::Utf8PersianCi => CharSet::Utf8,
            Self::Utf8EsperantoCi => CharSet::Utf8,
            Self::Utf8HungarianCi => CharSet::Utf8,
            Self::Utf8SinhalaCi => CharSet::Utf8,
            Self::Utf8German2Ci => CharSet::Utf8,
            Self::Utf8CroatianCi => CharSet::Utf8,
            Self::Utf8Unicode520Ci => CharSet::Utf8,
            Self::Utf8VietnameseCi => CharSet::Utf8,
            Self::Utf8GeneralMysql500Ci => CharSet::Utf8,
            Self::Utf8Mb4GeneralCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Bin => CharSet::Utf8Mb4,
            Self::Utf8Mb4UnicodeCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4IcelandicCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4LatvianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4RomanianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4SlovenianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4PolishCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4EstonianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4SpanishCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4SwedishCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4TurkishCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4CzechCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4DanishCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4LithuanianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4SlovakCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Spanish2Ci => CharSet::Utf8Mb4,
            Self::Utf8Mb4RomanCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4PersianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4EsperantoCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4HungarianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4SinhalaCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4German2Ci => CharSet::Utf8Mb4,
            Self::Utf8Mb4CroatianCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Unicode520Ci => CharSet::Utf8Mb4,
            Self::Utf8Mb4VietnameseCi => CharSet::Utf8Mb4,
            Self::Utf8Mb40900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4DePb0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Is0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Lv0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Ro0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Sl0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Pl0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Et0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Es0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Sv0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Tr0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Cs0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Da0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Lt0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Sk0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4EsTrad0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4La0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Eo0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Hu0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Hr0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Vi0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb40900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4DePb0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Is0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Lv0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Ro0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Sl0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Pl0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Et0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Es0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Sv0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Tr0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Cs0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Da0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Lt0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Sk0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4EsTrad0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4La0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Eo0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Hu0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Hr0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Vi0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Ja0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Ja0900AsCsKs => CharSet::Utf8Mb4,
            Self::Utf8Mb40900AsCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Ru0900AiCi => CharSet::Utf8Mb4,
            Self::Utf8Mb4Ru0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb4Zh0900AsCs => CharSet::Utf8Mb4,
            Self::Utf8Mb40900Bin => CharSet::Utf8Mb4,
            Self::Unknown(unknown) => CharSet::Unknown(unknown.to_owned()),
        }
    }

    pub fn unknown_to_string(&self) -> &String {
        match self {
            Self::Unknown(custom) => custom,
            _ => panic!("not Unknown"),
        }
    }

    pub fn string_to_unknown(string: &str) -> Option<Self> {
        Some(Self::Unknown(string.to_string()))
    }
}
