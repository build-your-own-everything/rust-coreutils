use assert_cmd::Command;

#[test]
fn runs() {
    let mut cmd = Command::cargo_bin("hello").expect("cmd should work");
    cmd.assert().success().stdout("Hello, world!\n");
}

#[test]
fn is_true() {
    let mut cmd = Command::cargo_bin("true").expect("should find true");
    cmd.assert().success();
}

#[test]
fn is_false() {
    let mut cmd = Command::cargo_bin("false").expect("should find false");
    cmd.assert().failure();
}
