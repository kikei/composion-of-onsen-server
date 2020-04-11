extern crate mecab;

use mecab::{Tagger, Node};
/**
 * MeCab: Yey Another Part-of-Speech and Morphological Analyzer
 * http://taku910.github.io/mecab/
 *
 * mecab-rs/mecab.rs
 * https://github.com/tsurai/mecab-rs/blob/master/src/mecab.rs
 */

fn node_to_yomi(node: &Node) -> String {
    /*
     * 0: 品詞, 1: 品詞細分類1, 2: 品詞細分類2, 3: 品詞細分類3,
     * 4: 活用型, 5: 活用形, 6: 原形, 7: 読み, 8: 発音
     */
    println!("node");
    String::from(match node.stat as i32 {
        mecab::MECAB_BOS_NODE => "",
        mecab::MECAB_EOS_NODE => "",
        _ => {
            let feature = &node.feature;
            let parts: Vec<&str> = feature.split(',').collect();
            println!("parts: {:?}", &parts);
            match parts {
                _ if parts.len() <= 7 || parts[7] == "" =>
                    &node.surface[..node.length as usize],
                _ => parts[7]
            }
        }
    })
}

#[derive(Clone, Copy)]
enum Pron {
    Single(&'static str),
    Sokuon(&'static str),
    Others(&'static str),
    Hatsuon(&'static str)
}

struct KatakanaAlphabet {
    katakana: &'static str,
    alphabet: Pron,
}

fn first_char(t: &str) -> Option<char> {
    t.chars().next()
}

fn alpha_to_sokuon(t: &str) -> String {
    match first_char(t) {
        Some(c) => format!("{}{}", c, t),
        None => String::from("")
    }
}

fn read_katakana<'a>(map: &'a Vec<KatakanaAlphabet>, text: &str)
                 -> Option<&'a KatakanaAlphabet> {
    match map.iter().find(|ka| text.starts_with(&ka.katakana)) {
            Some(ka) => Some(ka),
            None => None
        }
}

/*
fn dump_mecab_node(node: &Node) {
    println!("Node \
              id: {}, surface: {}, feature: {}, \
              length: {}, rlength: {}, \
              rcattr: {}, lcattr: {}, posid: {},\
              char_type: {}, stat: {}, isbest: {}, alpha: {}, \
              beta: {}, prob: {}, cost: {}",
             node.id,
             node.surface,
             node.feature,
             node.length,
             node.rlength,
             node.rcattr,
             node.lcattr,
             node.posid,
             node.char_type,
             node.stat,
             node.isbest,
             node.alpha,
             node.beta,
             node.prob,
             node.cost);
}
*/

fn tokenize_japanese(text: &String) -> Vec<String> {
    let mut tagger = Tagger::new("");
    let t = text.as_bytes();
    let nodes = tagger.parse_to_node(t);
    nodes.iter_next().map(|n| node_to_yomi(&n)).collect()
}

fn tokenize_katakana(text: &String) -> Vec<Pron> {
    let map = katakana_alphabet_map();
    let mut v = vec![];
    let mut i = 0;
    while i < text.len() {
        let t = read_katakana(&map, &text[i..]);
        v.push(match t {
            Some(ka) => {
                i += ka.katakana.len();
                ka.alphabet
            },
            None => {
                i += first_char(&text[i..]).map_or(3, |c| c.to_string().len());
                Pron::Others("-")
            }
        })
    }
    v
}

fn katakana_yomis(pron: Vec<Pron>) -> Vec<String> {
    let mut v = vec![];
    let mut last: Option<Pron> = None;
    for p in pron {
        match last {
            None => match p {
                Pron::Single(s1) |
                Pron::Others(s1) |
                Pron::Hatsuon(s1) => v.push(String::from(s1)),
                Pron::Sokuon(_) => last = Some(p),
            },
            Some(Pron::Sokuon(s0)) => match p {
                Pron::Single(s1) => {
                    v.push(alpha_to_sokuon(s1));
                    last = None;
                },
                Pron::Others(s1) => {
                    v.push(String::from(s1));
                    last = None;
                },
                Pron::Sokuon(_) => last = Some(p),
                Pron::Hatsuon(s1) => {
                    v.push(String::from(s0));
                    v.push(String::from(s1));
                    last = None;
                }
            },
            _ => (),
        }
    }
    v
}

fn katakana_alphabet_map() -> Vec<KatakanaAlphabet> {
    let map = vec![
        ("ウィ", Pron::Single("wi")),
        ("ウェ", Pron::Single("we")),
        ("キャ", Pron::Single("kya")),
        ("キュ", Pron::Single("kyu")),
        ("キョ", Pron::Single("kyo")),
        ("シャ", Pron::Single("sha")),
        ("シュ", Pron::Single("shu")),
        ("ショ", Pron::Single("sho")),
        ("チャ", Pron::Single("cha")),
        ("チュ", Pron::Single("chu")),
        ("チェ", Pron::Single("che")),
        ("チョ", Pron::Single("cho")),
        ("ニャ", Pron::Single("nya")),
        ("ニュ", Pron::Single("nyu")),
        ("ニョ", Pron::Single("nyo")),
        ("ヒャ", Pron::Single("hya")),
        ("ヒュ", Pron::Single("hyu")),
        ("ヒョ", Pron::Single("hyo")),
        ("ミャ", Pron::Single("mya")),
        ("ミュ", Pron::Single("myu")),
        ("ミョ", Pron::Single("myo")),
        ("リャ", Pron::Single("rya")),
        ("リュ", Pron::Single("ryu")),
        ("リョ", Pron::Single("ryo")),
        ("ヴァ", Pron::Single("va")),
        ("ヴィ", Pron::Single("vi")),
        ("ヴェ", Pron::Single("ve")),
        ("ヴォ", Pron::Single("vo")),
        ("ギャ", Pron::Single("gya")),
        ("ギュ", Pron::Single("gyu")),
        ("ギョ", Pron::Single("gya")),
        ("ジャ", Pron::Single("ja")),
        ("ジュ", Pron::Single("ju")),
        ("ジェ", Pron::Single("je")),
        ("ジョ", Pron::Single("jo")),
        ("ヂャ", Pron::Single("dya")),
        ("ヂュ", Pron::Single("dyu")),
        ("ヂェ", Pron::Single("dye")),
        ("ヂョ", Pron::Single("dyo")),
        ("ビャ", Pron::Single("bya")),
        ("ビュ", Pron::Single("byu")),
        ("ビェ", Pron::Single("bye")),
        ("ビョ", Pron::Single("byo")),
        ("ア", Pron::Single("a")),
        ("イ", Pron::Single("i")),
        ("ウ", Pron::Single("u")),
        ("エ", Pron::Single("e")),
        ("オ", Pron::Single("o")),
        ("カ", Pron::Single("ka")),
        ("キ", Pron::Single("ki")),
        ("ク", Pron::Single("ku")),
        ("ケ", Pron::Single("ke")),
        ("コ", Pron::Single("ko")),
        ("サ", Pron::Single("sa")),
        ("シ", Pron::Single("si")),
        ("ス", Pron::Single("su")),
        ("セ", Pron::Single("se")),
        ("ソ", Pron::Single("so")),
        ("タ", Pron::Single("ta")),
        ("チ", Pron::Single("chi")),
        ("ツ", Pron::Single("tsu")),
        ("テ", Pron::Single("te")),
        ("ト", Pron::Single("to")),
        ("ナ", Pron::Single("na")),
        ("ニ", Pron::Single("ni")),
        ("ヌ", Pron::Single("nu")),
        ("ネ", Pron::Single("ne")),
        ("ノ", Pron::Single("no")),
        ("ハ", Pron::Single("ha")),
        ("ヒ", Pron::Single("hi")),
        ("フ", Pron::Single("fu")),
        ("ヘ", Pron::Single("he")),
        ("ホ", Pron::Single("ho")),
        ("マ", Pron::Single("ma")),
        ("ミ", Pron::Single("mi")),
        ("ム", Pron::Single("mu")),
        ("メ", Pron::Single("me")),
        ("モ", Pron::Single("mo")),
        ("ヤ", Pron::Single("ya")),
        ("ユ", Pron::Single("yu")),
        ("ヨ", Pron::Single("yo")),
        ("ラ", Pron::Single("ra")),
        ("リ", Pron::Single("ri")),
        ("ル", Pron::Single("ru")),
        ("レ", Pron::Single("re")),
        ("ロ", Pron::Single("ro")),
        ("ワ", Pron::Single("wa")),
        ("ヲ", Pron::Single("wo")),
        ("ガ", Pron::Single("ga")),
        ("ギ", Pron::Single("gi")),
        ("グ", Pron::Single("gu")),
        ("ゲ", Pron::Single("ge")),
        ("ゴ", Pron::Single("go")),
        ("ザ", Pron::Single("za")),
        ("ジ", Pron::Single("ji")),
        ("ズ", Pron::Single("zu")),
        ("ゼ", Pron::Single("ze")),
        ("ゾ", Pron::Single("zo")),
        ("ダ", Pron::Single("da")),
        ("ヂ", Pron::Single("di")),
        ("ヅ", Pron::Single("du")),
        ("デ", Pron::Single("de")),
        ("ド", Pron::Single("do")),
        ("バ", Pron::Single("ba")),
        ("ビ", Pron::Single("bi")),
        ("ブ", Pron::Single("bu")),
        ("ベ", Pron::Single("be")),
        ("ボ", Pron::Single("bo")),
        ("パ", Pron::Single("pa")),
        ("ピ", Pron::Single("pi")),
        ("プ", Pron::Single("pu")),
        ("ペ", Pron::Single("pe")),
        ("ポ", Pron::Single("po")),
        ("ー", Pron::Others("-")),
        ("ン", Pron::Hatsuon("n")),
        ("ッ", Pron::Sokuon("xtu")),
        ("0", Pron::Others("0")),
        ("1", Pron::Others("1")),
        ("2", Pron::Others("2")),
        ("3", Pron::Others("3")),
        ("4", Pron::Others("4")),
        ("5", Pron::Others("5")),
        ("6", Pron::Others("6")),
        ("7", Pron::Others("7")),
        ("8", Pron::Others("8")),
        ("9", Pron::Others("9")),
    ];
    let v =
        map.iter()
        .map(|(k, a)| KatakanaAlphabet { katakana: k, alphabet: *a })
        .collect::<Vec<KatakanaAlphabet>>();
    v
}

pub fn scrub(text: &String) -> String {
    // let mut yomis = vec![];
    let katakanas: Vec<String> = tokenize_japanese(&text);
    let katakanas: String = katakanas.join("");
    let katakanas: Vec<Pron> = tokenize_katakana(&katakanas);
    let yomis: Vec<String> = katakana_yomis(katakanas);
    yomis.join("")
}
