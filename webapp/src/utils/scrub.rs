extern crate mecab;

use mecab::{Tagger, Node};
/**
 * MeCab: Yey Another Part-of-Speech and Morphological Analyzer
 * http://taku910.github.io/mecab/
 *
 * mecab-rs/mecab.rs
 * https://github.com/tsurai/mecab-rs/blob/master/src/mecab.rs
 */

fn print_node(node: &Node) {
    println!("fe: {}, rc: {}, lc: {}, po: {}, ct: {}, st: {}, be: {}, al: {}, be: {}, pr: {}, co: {}",
             node.feature,
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

fn node_to_yomi(node: &Node) -> String {
    /*
     * 0: 品詞, 1: 品詞細分類1, 2: 品詞細分類2, 3: 品詞細分類3,
     * 4: 活用型, 5: 活用形, 6: 原形, 7: 読み, 8: 発音
     */
    // print_node(&node);
    String::from(match node.stat as i32 {
        mecab::MECAB_BOS_NODE => "",
        mecab::MECAB_EOS_NODE => "",
        _ => {
            let feature = &node.feature;
            let parts: Vec<&str> = feature.split(',').collect();
            println!("parts: {:?}", &parts);
            match parts {
                _ if parts.len() <= 6 || parts[6] == "" =>
                    &node.surface[..node.length as usize],
                _ => parts[6]
            }
        }
    })
}

#[derive(Debug, Clone)]
enum Pron {
    Single(String),
    Sokuon(String),
    Others(String),
    Hatsuon(String)
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
                ka.alphabet.clone()
            },
            None => {
                match first_char(&text[i..]) {
                    Some(c) => {
                        i += c.to_string().len();
                        if c.is_alphanumeric() {
                            Pron::Single(c.to_string())
                        } else {
                            Pron::Others("-".to_string())
                        }
                    },
                    None => {
                        i += 3;
                        Pron::Others("-".to_string())
                    }
                }
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
                    v.push(alpha_to_sokuon(&s1.as_str()));
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
        ("ウィ", Pron::Single("wi".to_string())),
        ("ウェ", Pron::Single("we".to_string())),
        ("キャ", Pron::Single("kya".to_string())),
        ("キュ", Pron::Single("kyu".to_string())),
        ("キョ", Pron::Single("kyo".to_string())),
        ("シャ", Pron::Single("sha".to_string())),
        ("シュ", Pron::Single("shu".to_string())),
        ("ショ", Pron::Single("sho".to_string())),
        ("チャ", Pron::Single("cha".to_string())),
        ("チュ", Pron::Single("chu".to_string())),
        ("チェ", Pron::Single("che".to_string())),
        ("チョ", Pron::Single("cho".to_string())),
        ("ニャ", Pron::Single("nya".to_string())),
        ("ニュ", Pron::Single("nyu".to_string())),
        ("ニョ", Pron::Single("nyo".to_string())),
        ("ヒャ", Pron::Single("hya".to_string())),
        ("ヒュ", Pron::Single("hyu".to_string())),
        ("ヒョ", Pron::Single("hyo".to_string())),
        ("ミャ", Pron::Single("mya".to_string())),
        ("ミュ", Pron::Single("myu".to_string())),
        ("ミョ", Pron::Single("myo".to_string())),
        ("リャ", Pron::Single("rya".to_string())),
        ("リュ", Pron::Single("ryu".to_string())),
        ("リョ", Pron::Single("ryo".to_string())),
        ("ヴァ", Pron::Single("va".to_string())),
        ("ヴィ", Pron::Single("vi".to_string())),
        ("ヴェ", Pron::Single("ve".to_string())),
        ("ヴォ", Pron::Single("vo".to_string())),
        ("ギャ", Pron::Single("gya".to_string())),
        ("ギュ", Pron::Single("gyu".to_string())),
        ("ギョ", Pron::Single("gya".to_string())),
        ("ジャ", Pron::Single("ja".to_string())),
        ("ジュ", Pron::Single("ju".to_string())),
        ("ジェ", Pron::Single("je".to_string())),
        ("ジョ", Pron::Single("jo".to_string())),
        ("ヂャ", Pron::Single("dya".to_string())),
        ("ヂュ", Pron::Single("dyu".to_string())),
        ("ヂェ", Pron::Single("dye".to_string())),
        ("ヂョ", Pron::Single("dyo".to_string())),
        ("ビャ", Pron::Single("bya".to_string())),
        ("ビュ", Pron::Single("byu".to_string())),
        ("ビェ", Pron::Single("bye".to_string())),
        ("ビョ", Pron::Single("byo".to_string())),
        ("ア", Pron::Single("a".to_string())),
        ("イ", Pron::Single("i".to_string())),
        ("ウ", Pron::Single("u".to_string())),
        ("エ", Pron::Single("e".to_string())),
        ("オ", Pron::Single("o".to_string())),
        ("カ", Pron::Single("ka".to_string())),
        ("キ", Pron::Single("ki".to_string())),
        ("ク", Pron::Single("ku".to_string())),
        ("ケ", Pron::Single("ke".to_string())),
        ("コ", Pron::Single("ko".to_string())),
        ("サ", Pron::Single("sa".to_string())),
        ("シ", Pron::Single("si".to_string())),
        ("ス", Pron::Single("su".to_string())),
        ("セ", Pron::Single("se".to_string())),
        ("ソ", Pron::Single("so".to_string())),
        ("タ", Pron::Single("ta".to_string())),
        ("チ", Pron::Single("chi".to_string())),
        ("ツ", Pron::Single("tsu".to_string())),
        ("テ", Pron::Single("te".to_string())),
        ("ト", Pron::Single("to".to_string())),
        ("ナ", Pron::Single("na".to_string())),
        ("ニ", Pron::Single("ni".to_string())),
        ("ヌ", Pron::Single("nu".to_string())),
        ("ネ", Pron::Single("ne".to_string())),
        ("ノ", Pron::Single("no".to_string())),
        ("ハ", Pron::Single("ha".to_string())),
        ("ヒ", Pron::Single("hi".to_string())),
        ("フ", Pron::Single("fu".to_string())),
        ("ヘ", Pron::Single("he".to_string())),
        ("ホ", Pron::Single("ho".to_string())),
        ("マ", Pron::Single("ma".to_string())),
        ("ミ", Pron::Single("mi".to_string())),
        ("ム", Pron::Single("mu".to_string())),
        ("メ", Pron::Single("me".to_string())),
        ("モ", Pron::Single("mo".to_string())),
        ("ヤ", Pron::Single("ya".to_string())),
        ("ユ", Pron::Single("yu".to_string())),
        ("ヨ", Pron::Single("yo".to_string())),
        ("ラ", Pron::Single("ra".to_string())),
        ("リ", Pron::Single("ri".to_string())),
        ("ル", Pron::Single("ru".to_string())),
        ("レ", Pron::Single("re".to_string())),
        ("ロ", Pron::Single("ro".to_string())),
        ("ワ", Pron::Single("wa".to_string())),
        ("ヲ", Pron::Single("wo".to_string())),
        ("ガ", Pron::Single("ga".to_string())),
        ("ギ", Pron::Single("gi".to_string())),
        ("グ", Pron::Single("gu".to_string())),
        ("ゲ", Pron::Single("ge".to_string())),
        ("ゴ", Pron::Single("go".to_string())),
        ("ザ", Pron::Single("za".to_string())),
        ("ジ", Pron::Single("ji".to_string())),
        ("ズ", Pron::Single("zu".to_string())),
        ("ゼ", Pron::Single("ze".to_string())),
        ("ゾ", Pron::Single("zo".to_string())),
        ("ダ", Pron::Single("da".to_string())),
        ("ヂ", Pron::Single("di".to_string())),
        ("ヅ", Pron::Single("du".to_string())),
        ("デ", Pron::Single("de".to_string())),
        ("ド", Pron::Single("do".to_string())),
        ("バ", Pron::Single("ba".to_string())),
        ("ビ", Pron::Single("bi".to_string())),
        ("ブ", Pron::Single("bu".to_string())),
        ("ベ", Pron::Single("be".to_string())),
        ("ボ", Pron::Single("bo".to_string())),
        ("パ", Pron::Single("pa".to_string())),
        ("ピ", Pron::Single("pi".to_string())),
        ("プ", Pron::Single("pu".to_string())),
        ("ペ", Pron::Single("pe".to_string())),
        ("ポ", Pron::Single("po".to_string())),
        ("ー", Pron::Others("-".to_string())),
        ("ン", Pron::Hatsuon("n".to_string())),
        ("ッ", Pron::Sokuon("xtu".to_string())),
        ("0", Pron::Others("0".to_string())),
        ("1", Pron::Others("1".to_string())),
        ("2", Pron::Others("2".to_string())),
        ("3", Pron::Others("3".to_string())),
        ("4", Pron::Others("4".to_string())),
        ("5", Pron::Others("5".to_string())),
        ("6", Pron::Others("6".to_string())),
        ("7", Pron::Others("7".to_string())),
        ("8", Pron::Others("8".to_string())),
        ("9", Pron::Others("9".to_string()))
    ];
    let v =
        map.iter()
        .map(|(k, a)| KatakanaAlphabet { katakana: k, alphabet: a.clone() })
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
