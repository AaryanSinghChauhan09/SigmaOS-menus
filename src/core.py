"""Application menu database for SigmaOS-menus (inspired by cinnamon-menus
and mintmenu).

Principles: the menu is a pure in-memory database, deterministic search
ranking, fail-fast on unknown entries/fields, rebuild is idempotent, editing
is validated before it is stored, plan vs apply separation.
"""
import argparse, copy, json, os, sys, tempfile

ENGINE = {
    "name": "sigma-menus",
    "summary": "Menu database: categories, search, entry editing, rebuild",
    "language": "python",
    "verbs": ["categories", "search", "edit-entry", "rebuild"],
    "principles": ["Abstraction", "Least Astonishment", "Fail-Fast",
                   "KISS", "DRY", "Deterministic Testing"],
}

cli_doc = """\
$ sigma-menus categories
$ sigma-menus search fire
$ sigma-menus edit-entry firefox --name "Firefox Web Browser"
$ sigma-menus rebuild
"""

DEFAULT_STORE = os.path.expanduser("~/.config/sigma-menus/menu.json")
FIELDS = ("name", "exec", "categories", "keywords", "hidden")

DEMO_DB = {
    "firefox": {"name": "Firefox", "exec": "firefox %u",
                "categories": ["Network", "WebBrowser"], "keywords": ["internet", "web"]},
    "files": {"name": "Files", "exec": "nemo %U",
              "categories": ["System", "FileManager"], "keywords": ["folder"]},
    "terminal": {"name": "Terminal", "exec": "x-terminal-emulator",
                 "categories": ["System", "TerminalEmulator"], "keywords": ["shell"]},
    "calc": {"name": "Calculator", "exec": "sigma-calc",
             "categories": ["Utility", "Calculator"], "keywords": ["math"]},
}


class MenuError(ValueError):
    pass


def load_db(path):
    if not os.path.exists(path):
        return copy.deepcopy(DEMO_DB)
    with open(path, "r", encoding="utf-8") as fh:
        data = json.load(fh)
    for eid, e in data.items():  # fail-fast on corrupt store
        if not isinstance(e.get("name"), str) or not e["name"].strip():
            raise MenuError(f"entry {eid!r} has no name")
    return data


def save_db(path, db):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    tmp = path + ".sigma-tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(db, fh, indent=2, sort_keys=True)
    os.replace(tmp, path)


def categories(db):
    """Sorted (category, count) pairs over visible entries. Pure."""
    counts = {}
    for e in db.values():
        if e.get("hidden"):
            continue
        for c in e.get("categories", []):
            counts[c] = counts.get(c, 0) + 1
    return sorted(counts.items())


def search(db, query):
    """Rank visible entries: exact < prefix < substring < keyword < category.

    Ties break alphabetically by name (deterministic). Pure.
    """
    q = str(query).strip().lower()
    if not q:
        raise MenuError("empty query")
    scored = []
    for eid, e in db.items():
        if e.get("hidden"):
            continue
        name = e["name"].lower()
        if name == q:
            score = 0
        elif name.startswith(q):
            score = 1
        elif q in name:
            score = 2
        elif any(q in k.lower() for k in e.get("keywords", [])):
            score = 3
        elif any(q in c.lower() for c in e.get("categories", [])):
            score = 4
        else:
            continue
        scored.append((score, name, eid))
    scored.sort()
    return [{"score": s, "id": eid, "name": db[eid]["name"]} for s, _, eid in scored]


def edit_entry(db, entry_id, field, value):
    """Validate then mutate one field of one entry (fail-fast)."""
    if entry_id not in db:
        raise MenuError(f"unknown entry: {entry_id}")
    if field not in FIELDS:
        raise MenuError(f"unknown field: {field} (have: {', '.join(FIELDS)})")
    if field in ("categories", "keywords"):
        value = [v.strip() for v in str(value).split(",") if v.strip()]
        if not value:
            raise MenuError(f"{field} must be a comma-separated list")
    elif field == "hidden":
        value = str(value).lower() in ("1", "true", "yes", "on")
    else:
        value = str(value).strip()
        if not value:
            raise MenuError(f"{field} must be non-empty")
    db[entry_id][field] = value
    return db


def rebuild(db):
    """Deterministic menu tree: category -> entry ids sorted by name. Pure."""
    tree = {}
    for eid, e in db.items():
        if e.get("hidden"):
            continue
        for cat in e.get("categories", ["Other"]):
            tree.setdefault(cat, []).append((e["name"].lower(), eid))
    return {cat: [eid for _, eid in sorted(pairs)]
            for cat, pairs in sorted(tree.items())}


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog=ENGINE["name"], description=ENGINE["summary"])
    p.add_argument("--store", default=DEFAULT_STORE, help="menu store (JSON)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("categories", help="list categories with entry counts")
    s = sub.add_parser("search", help="search entries with ranking")
    s.add_argument("query")
    e = sub.add_parser("edit-entry", help="edit one field of an entry")
    e.add_argument("entry_id")
    e.add_argument("--name")
    e.add_argument("--exec", dest="exec_")
    e.add_argument("--categories")
    e.add_argument("--keywords")
    e.add_argument("--hidden")
    sub.add_parser("rebuild", help="rebuild the category tree (pure)")
    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    try:
        db = load_db(args.store)
        if args.cmd == "categories":
            for cat, n in categories(db):
                print(f"{cat}\t{n}")
        elif args.cmd == "search":
            hits = search(db, args.query)
            if not hits:
                print("no matches")
            for h in hits:
                print(f"{h['score']} {h['id']}\t{h['name']}")
        elif args.cmd == "edit-entry":
            patch = {"name": args.name, "exec": args.exec_,
                     "categories": args.categories, "keywords": args.keywords,
                     "hidden": args.hidden}
            fields = {k: v for k, v in patch.items() if v is not None}
            if not fields:
                print("error: nothing to edit", file=sys.stderr)
                return 1
            for k, v in fields.items():
                edit_entry(db, args.entry_id, k, v)
            save_db(args.store, db)
            print(f"edited {args.entry_id}: {fields}")
        elif args.cmd == "rebuild":
            tree = rebuild(db)
            for cat, ids in tree.items():
                print(f"{cat}: {', '.join(ids)}")
        return 0
    except (MenuError, OSError, ValueError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1


def self_test() -> None:
    db = copy.deepcopy(DEMO_DB)
    # categories (pure, sorted)
    assert categories(db) == [("Calculator", 1), ("FileManager", 1), ("Network", 1),
                              ("System", 2), ("TerminalEmulator", 1), ("Utility", 1),
                              ("WebBrowser", 1)]
    # search ranking (deterministic)
    assert [h["id"] for h in search(db, "calc")] == ["calc"]          # exact
    assert [h["id"] for h in search(db, "fire")] == ["firefox"]       # prefix
    assert search(db, "calc")[0]["score"] == 1
    sys_hits = [h["id"] for h in search(db, "system")]
    assert sys_hits == ["files", "terminal"], sys_hits                # tie: alpha
    assert all(h["score"] == 4 for h in search(db, "system"))
    assert search(db, "folder")[0]["id"] == "files"                   # keyword
    assert search(db, "zzz") == []
    try:
        search(db, "  ")
    except MenuError:
        pass
    else:
        raise AssertionError("empty query not caught")
    # edit-entry: validate, mutate, hide
    edit_entry(db, "firefox", "name", "Firefox Web Browser")
    assert db["firefox"]["name"] == "Firefox Web Browser"
    edit_entry(db, "calc", "hidden", "true")
    assert db["calc"]["hidden"] is True
    assert "calc" not in [h["id"] for h in search(db, "calc")]
    for bad in (lambda: edit_entry(db, "ghost", "name", "Ghost"),
                lambda: edit_entry(db, "files", "colour", "red"),
                lambda: edit_entry(db, "files", "name", "   "),
                lambda: edit_entry(db, "files", "categories", ",")):
        try:
            bad()
        except MenuError:
            pass
        else:
            raise AssertionError("expected MenuError")
    # rebuild: deterministic, hidden excluded, idempotent
    tree = rebuild(db)
    assert tree["System"] == ["files", "terminal"]
    assert "calc" not in tree.get("Calculator", [])
    assert tree == rebuild(db)
    assert list(tree) == sorted(tree)
    # persistence round-trip (tempdir only)
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "menu.json")
        save_db(path, db)
        again = load_db(path)
        assert again["firefox"]["name"] == "Firefox Web Browser"
        assert rebuild(again) == tree


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
