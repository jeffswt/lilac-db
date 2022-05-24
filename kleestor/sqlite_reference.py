
import os
import sqlite3
import time


def main():
    path = '/tmp/sqlite3.db'
    if os.path.exists(path):
        os.remove(path)
    conn = sqlite3.connect(path)

    max_counter = 500000
    total_bytes = 0

    ##########################################################################
    # write

    tm = time.time()
    cur = conn.cursor()
    cur.execute('CREATE TABLE data (k TEXT PRIMARY KEY, v TEXT);')

    for j in range(0, max_counter, 100):
        kv_pairs = []
        for _i in range(j, j + 100):
            i = (_i * 998244353) % max_counter
            # i = _i
            key = f'sample-key-{i}'
            value = f'value-{i}-0123456789abcde-0123456789abcde-0123456789abcde-0123456789abcde-{i}'
            total_bytes += len(key)
            total_bytes += len(value)
            kv_pairs.append((key, value))
        cur.executemany('INSERT INTO data (k, v) VALUES (?, ?);', kv_pairs)

    conn.commit()
    duration = time.time() - tm
    print(f'write: {int(max_counter / duration)} transactions per sec')
    print(f'write: {int(total_bytes / duration)} bytes per sec')

    ##########################################################################
    # read

    tm = time.time()
    cur = conn.cursor()
    cur.execute('SELECT k, v FROM data ORDER BY k ASC;')

    while True:
        items = cur.fetchmany(97)
        if len(items) < 97:
            break

    duration = time.time() - tm
    print(f'seqscan: {int(max_counter / duration)} transactions per sec')
    print(f'seqscan: {int(total_bytes / duration)} bytes per sec')

    ##########################################################################
    # seq read

    tm = time.time()

    for i in range(0, max_counter):
        cur = conn.cursor()
        key = f'sample-key-{i}'
        cur.execute('SELECT v FROM data WHERE k = ?;', [key])
        value = cur.fetchone()[0]

    duration = time.time() - tm
    print(f'seqread: {int(max_counter / duration)} transactions per sec')
    print(f'seqread: {int(total_bytes / duration)} bytes per sec')

    ##########################################################################
    # rand read

    tm = time.time()

    for _i in range(0, max_counter):
        i = (_i * 998244353) % max_counter
        cur = conn.cursor()
        key = f'sample-key-{i}'
        cur.execute('SELECT v FROM data WHERE k = ?;', [key])
        value = cur.fetchone()[0]

    duration = time.time() - tm
    print(f'randread: {int(max_counter / duration)} transactions per sec')
    print(f'randread: {int(total_bytes / duration)} bytes per sec')

    conn.close()
    os.remove(path)
    return


if __name__ == '__main__':
    main()
