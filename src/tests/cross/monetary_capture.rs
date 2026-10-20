// Opt-in, ordered diagnostic capture. Independent of oracle blobs/digests.
use super::*;
use serde_json::json;

std::thread_local! {
    static SEQUENCE: Cell<u64> = const { Cell::new(0) };
    // Oracle figure replay names each observation by its corpus input digest.
    static ORIGIN: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}
static STREAM: OnceLock<Option<std::sync::Mutex<std::fs::File>>> = OnceLock::new();

// Observations made while replaying one oracle input, in the same capture format as a
// test, with that input's digest in place of the test name.
pub(super) fn replay_as<T>(origin: &str, replay: impl FnOnce() -> T) -> T {
    ORIGIN.set(Some(origin.to_string()));
    SEQUENCE.set(0);
    let value = replay();
    ORIGIN.set(None);
    value
}

fn record(kind: &str, money: serde_json::Value, debug: String) {
    let origin = ORIGIN.with_borrow(Clone::clone);
    if origin.is_none() && REPLAYING.get() {
        return;
    }
    let Some(stream) = STREAM.get_or_init(|| {
        std::env::var_os("TAX_FIGURE_CAPTURE").map(|value| {
            let root = PathBuf::from(value);
            assert!(root.is_absolute(), "TAX_FIGURE_CAPTURE must be absolute");
            fs::create_dir_all(&root).unwrap();
            // Never append a second test run to existing evidence.
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("events.jsonl"))
                .unwrap();
            std::sync::Mutex::new(file)
        })
    }) else {
        return;
    };
    let thread = std::thread::current();
    let test = match &origin {
        Some(origin) => origin.as_str(),
        None => thread.name().expect("figure capture requires a named test thread"),
    };
    assert!(origin.is_some() || test.contains("::"), "figure capture requires a test origin");
    let index = SEQUENCE.get();
    SEQUENCE.set(index.checked_add(1).unwrap());
    // Serialize before acquiring the shared file: JSON escaping otherwise
    // makes many small writes while every other test waits for this lock.
    let bytes = serde_json::to_vec(&json!({
        "version": 1, "test": test, "index": index, "kind": kind,
        "money": money, "debug": debug,
    }))
    .unwrap();
    let mut file = stream.lock().unwrap();
    file.write_all(&bytes).unwrap();
    file.write_all(b"\n").unwrap();
}

pub(super) fn estimate(result: &FederalResult) {
    if std::env::var_os("TAX_FIGURE_CAPTURE").is_none() {
        return;
    }
    let lines: Vec<_> = result
        .lines
        .iter()
        .map(|line| {
            json!({
                "id": line.id, "kind": format!("{:?}", line.kind), "amount": line.amount,
            })
        })
        .collect();
    let return_rows: Vec<_> = result
        .return_lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            json!({
                "index": index, "amount": line.amount, "money": line.money,
                "form_line": format!("{:?}", line.form_line),
            })
        })
        .collect();
    record(
        "Estimate",
        json!({
            "top": {
                "tax": result.tax, "refundable_credits": result.refundable_credits,
                "payments": result.payments, "refund": result.refund, "balance": result.balance,
                "farmland_installments.applicable": result.farmland_installments.as_ref().map(|v| v.applicable),
                "farmland_installments.first": result.farmland_installments.as_ref().map(|v| v.first),
                "farmland_installments.remaining": result.farmland_installments.as_ref().map(|v| v.remaining),
                "farmland_installments.current": result.farmland_installments.as_ref().map(|v| v.current),
            },
            "lines": lines,
            "return_rows": return_rows,
        }),
        format!("{result:?}"),
    );
}

pub(super) fn send(session: &Interview, reply: &Reply) {
    if std::env::var_os("TAX_FIGURE_CAPTURE").is_none() {
        return;
    }
    // Match every Send output owner: reply plus the saved session values
    // whose Avro digest the oracle otherwise records without retaining them.
    record(
        "Send",
        serde_json::Value::Null,
        format!(
            "SendOutput {{ reply: {reply:?}, draft: {:?}, user: {:?}, flags: {:?} }}",
            session.draft,
            session.user,
            session.oracle_flags(),
        ),
    );
}
