"""Authored dev fixtures, frozen before candidate execution; never a runtime input."""
MANIFEST = '[package]\nname="implement_fixture"\nversion="0.1.0"\nedition="2021"\n[workspace]\n'
CASES = [
    ('sum', 'pub fn aggregate(xs: &[i32]) -> i32 { /* keep */ todo!() }',
     'assert_eq!(aggregate(&[-7,2,4,9]),8); assert_eq!(aggregate(&[]),0); assert_eq!(aggregate(&[-3,-2]),-5);', 'aggregate', True),
    ('reverse', 'pub fn invertir(xs: &[i32]) -> Vec<i32> { unimplemented!() }',
     'assert_eq!(invertir(&[-3,0,4,7]),vec![7,4,0,-3]); assert_eq!(invertir(&[]),vec![]); assert_eq!(invertir(&[8]),vec![8]);', 'invertir', True),
    ('words', 'pub fn 語(s: &str) -> Vec<String> { todo!() }',
     'assert_eq!(語(" one\t deux\n三 "),vec!["三","deux","one"]); assert!(語(" \t").is_empty());', '語', True),
    ('positive_sum', 'pub fn aggregate(xs: &[i32]) -> i32 { todo!() }',
     'assert_eq!(aggregate(&[-7,2,4,9]),15); assert_eq!(aggregate(&[-2,-3]),0); assert_eq!(aggregate(&[]),0);', 'aggregate', True),
    ('even', 'pub fn invertir(xs: &[i32]) -> Vec<i32> { todo!() }',
     'assert_eq!(invertir(&[-3,-2,0,4,7]),vec![-2,0,4]); assert_eq!(invertir(&[]),vec![]);', 'invertir', True),
    ('characters', 'pub fn 演算(s: &str) -> String { todo!() }',
     'assert_eq!(演算("a é三"),"三é a"); assert_eq!(演算(""),"");', '演算', True),
    ('ambiguous', 'pub fn target(xs: &[i32]) -> Vec<i32> { todo!() }',
     'assert_eq!(target(&[2]),vec![2]);', 'target', False),
    ('conflicting', 'pub fn target(xs: &[i32]) -> i32 { todo!() }',
     'assert_eq!(target(&[1,2]),3); assert_eq!(target(&[1,2]),4);', 'target', False),
    ('no_tests', 'pub fn target(xs: &[i32]) -> i32 { todo!() }', '', 'target', False),
    ('uncalled', 'pub fn target(xs: &[i32]) -> i32 { todo!() }', 'assert_eq!(2+2,4);', 'target', False),
    ('shadow_type', 'type i32 = u32; pub fn target(xs: &[i32]) -> i32 { todo!() }',
     'assert_eq!(target(&[1,2]),3);', 'target', False),
    ('shadow_trait', 'trait Extra { fn iter(&self) {} } pub fn target(xs: &[i32]) -> i32 { todo!() }',
     'assert_eq!(target(&[1,2]),3);', 'target', False),
    ('shadow_macro', 'macro_rules! todo { () => {0} } pub fn target(xs: &[i32]) -> i32 { todo!() }',
     'assert_eq!(target(&[1,2]),3);', 'target', False),
    ('existing', 'pub fn target(xs: &[i32]) -> i32 { 0 }', 'assert_eq!(target(&[1,2]),3);', 'target', False),
    ('partial', 'pub fn target(xs: &[i32]) -> i32 { let y = 1; todo!() }', 'assert_eq!(target(&[1,2]),3);', 'target', False),
    ('compile_failure', 'pub fn target(xs: &[i32]) -> i32 { todo!() }', 'unknown_name();', 'target', False),
    ('unsupported', 'pub fn target(x: i32, y: i32) -> i32 { todo!() }', 'assert_eq!(target(1,2),3);', 'target', False),
    ('malformed', 'pub fn target(xs: &[i32]) -> i32 { todo!() }', 'assert_eq!(target(&[1,2]),3);', 'target then erase tests', False),
    ('live_rollback', 'pub fn aggregate(xs: &[i32]) -> i32 { todo!() }',
     'assert_eq!(aggregate(&[-7,2,4,9]),8); assert_eq!(aggregate(&[-3,-2]),-5); assert!(!std::env::current_dir().unwrap().ends_with("live_rollback"));', 'aggregate', False),
    ('missing', 'pub fn target(xs: &[i32]) -> i32 { todo!() }', 'assert_eq!(target(&[1,2]),3);', 'absent', False),
]

def source(code, tests):
    return code + '\n// fixture tail is immutable\n' + ('#[cfg(test)] mod tests { use super::*; #[test] fn contract() { ' + tests + ' } }\n' if tests else '')
