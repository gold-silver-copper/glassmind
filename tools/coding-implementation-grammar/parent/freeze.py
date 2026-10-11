from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent
cases=[]
def add(name,decl,test,target,kind='implement',request=None):
    source=decl+'\n// This source tail and its assertions must remain unchanged.\n'
    if test:source+='#[cfg(test)] mod tests { use super::*; #[test] fn contract() { '+test+' } }\n'
    cases.append(dict(name=name,source=source,target=target,kind=kind,request=request or f'implement `{target}`'))
add('sum_all','pub fn fold_values(data: &[i64]) -> i64 { todo!() }','assert_eq!(fold_values(&[-8,-3,0,2,7]),-2); assert_eq!(fold_values(&[]),0);','fold_values')
add('sum_negative','pub fn fold_values(data: &[i64]) -> i64 { todo!() }','assert_eq!(fold_values(&[-8,-3,0,2,7]),-11); assert_eq!(fold_values(&[4,9]),0);','fold_values')
add('reverse_odd','pub fn select_values(data: &[i32]) -> Vec<i32> { unimplemented!() }','assert_eq!(select_values(&[-5,-2,0,3,8,9]),vec![9,3,-5]); assert_eq!(select_values(&[]),vec![]);','select_values')
add('unicode_reverse','pub fn 並べる(data: &str) -> String { todo!() }','assert_eq!(並べる("ø 中 x"),"x 中 ø"); assert_eq!(並べる("a\tb"),"b\ta"); assert_eq!(並べる(""),"");','並べる')
add('words_forward','pub fn pieces(data: &str) -> Vec<String> { todo!() }','assert_eq!(pieces(" oak\t birch\n松 "),vec!["oak","birch","松"]); assert!(pieces(" \t").is_empty());','pieces')
add('count_chars','pub fn measure(data: &str) -> usize { todo!() }','assert_eq!(measure("é 中"),3); assert_eq!(measure("abc def"),7); assert_eq!(measure(""),0);','measure')
add('competing','pub fn fold_values(data: &[i64]) -> i64 { todo!() }','assert_eq!(fold_values(&[2]),2);','fold_values','ambiguous')
add('alias','type i32 = u32; pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(target(&[2,3]),5);','target','unchanged')
add('partial','pub fn target(data: &[i32]) -> i32 { let _keep = 1; todo!() }','assert_eq!(target(&[2,3]),5);','target','unchanged')
add('existing','pub fn target(data: &[i32]) -> i32 { 0 }','assert_eq!(target(&[2,3]),5);','target','unchanged')
add('macro_shadow','macro_rules! todo { () => {0} } pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(target(&[2,3]),5);','target','unchanged')
add('negative','pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(target(&[2,3]),5);','target','unchanged','please do not implement `target`')
add('compound','pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(target(&[2,3]),5);','target','unchanged','implement `target` and rename it to `new_name`')
add('behavior','pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(target(&[2,3]),5);','target','unchanged','implement `target` with wrapping arithmetic')
add('unexecuted','pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(2+3,5);','target','unchanged')
add('no_tests','pub fn target(data: &[i32]) -> i32 { todo!() }','','target','unchanged')
add('compiler_failure','pub fn target(data: &[i32]) -> i32 { todo!() }','missing_function();','target','unchanged')
add('conflicting','pub fn target(data: &[i32]) -> i32 { todo!() }','assert_eq!(target(&[2,3]),5); assert_eq!(target(&[2,3]),6);','target','unchanged')
(HERE/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
(HERE/'PLAN.md').write_text('''# Prospective implementation grammar parent gates

Current root baseline freshlypasses12 exact-name query/rename boundaries using
separate unambiguous function/field projects; freeze preservesobservedsourcebytes.
Do not repeat prior unsupportedbaselineassumptions. Candidate mustretainall12.

Eighteen fresh native contracts:6 implementations with differentbehaviors for
identicalsignatures andunrelated/Unicode names,1actualambiguouscase,11unchanged
controls. Preserveexactbytesoutsideentirestubbody, assertionsandmanifest.
Allpositiveoriginals and failingcontrols mustcompileandexecutefailingtests;
no-tests/unexecuted/compile-failure controls have explicit distinctbaselineconditions.
No liveCargo.lock orartifactadditiononrefusal, includingunsetCARGO_TARGET_DIR.
Nativebeforechecks use independentcopies so theydo not createfilesintheactionroot.
Everyaccepted body passesunchangednativechecks. Competingall-sum/positive-sum
bothpasspublictest[2] butgive8/11 on[-3,2,4,5]; confirmnative alternatives and
requirecomplete multi-passrefusal. Never treat uniquechoice assemanticproof.

Requirefull765>=218/all215oldpassesretained/zerofail/unchangeddenominator before
shipping. Runtimeproposer receivesonlycode signature; no request/docs/assertions
or identifier-based algorithmselection. No newgenerationfamily inthisexperiment.
Parentadaptersmayadaptwireinterfaceonly; frozencontracts/gatescannotchange.
''')
(HERE/'freeze.json').write_text(json.dumps({n:hashlib.sha256((HERE/n).read_bytes()).hexdigest() for n in ['cases.json','PLAN.md','freeze.py']},indent=2)+'\n')
print(json.dumps(dict(cases=len(cases),implement=sum(c['kind']=='implement' for c in cases),ambiguous=sum(c['kind']=='ambiguous' for c in cases),unchanged=sum(c['kind']=='unchanged' for c in cases))))
