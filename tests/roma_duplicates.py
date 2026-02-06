import sqlite3

def get_table_data(db_path, table_name):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Fetch column names to ensure schemas match
    cursor.execute(f"PRAGMA table_info({table_name})")
    columns = [info[1] for info in cursor.fetchall()]
    
    # Fetch all rows
    cursor.execute(f"SELECT * FROM {table_name}")
    data = cursor.fetchall()
    
    conn.close()
    return set(data), columns

def compare_sqlite_tables(db1_path, db2_path, table_name):
    print(f"Comparing table '{table_name}'...")
    
    # 1. Get data from both databases
    data1, cols1 = get_table_data(db1_path, table_name)
    data2, cols2 = get_table_data(db2_path, table_name)

    # 2. Check Schema
    if cols1 != cols2:
        print("❌ Schemas are different!")
        print(f"DB1 Cols: {cols1}\nDB2 Cols: {cols2}")
        return

    # 3. Compare using Set Operations
    # rows in DB1 but not in DB2
    only_in_1 = data1 - data2
    # rows in DB2 but not in DB1
    only_in_2 = data2 - data1

    if not only_in_1 and not only_in_2:
        print("✅ Tables are 100% identical.")
    else:
        print(f"❌ Differences found!")
        if only_in_1:
            print(f"Rows unique to DB1 ({len(only_in_1)}):", list(only_in_1))
        if only_in_2:
            print(f"Rows unique to DB2 ({len(only_in_2)}):", list(only_in_2))

# Usage
compare_sqlite_tables('output/text/en.sqlite', 'output/text/de.sqlite', 'character_names_roma')