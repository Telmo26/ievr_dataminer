import sqlite3

# ---- CONFIG ----
DB1_PATH = "data/characters.sqlite"
DB2_PATH = "data/text/en.sqlite"

DB1_TABLE = "characters"
DB1_COLUMN = "name_id"

DB2_TABLE = "character_names"
DB2_COLUMN = "id"
# ----------------

def main():
    conn1 = sqlite3.connect(DB1_PATH)
    conn2 = sqlite3.connect(DB2_PATH)

    cur1 = conn1.cursor()
    cur2 = conn2.cursor()

    # Read all lookup values from DB1
    cur1.execute(f"SELECT {DB1_COLUMN} FROM {DB1_TABLE}")
    values = [row[0] for row in cur1.fetchall()]

    print(f"Checking {len(values)} values...\n")

    found = []
    missing = []

    for value in values:
        cur2.execute(
            f"SELECT 1 FROM {DB2_TABLE} WHERE {DB2_COLUMN} = ? LIMIT 1",
            (value,)
        )
        if cur2.fetchone():
            found.append(value)
        else:
            missing.append(value)

    conn1.close()
    conn2.close()

    print(f"FOUND: {len(found)}")
    print(f"MISSING: {len(missing)}")

    # Optional: print missing values
    if missing:
        print("\nMissing values:")
        for v in missing:
            print(v)

if __name__ == "__main__":
    main()
