
import matplotlib
import matplotlib.pyplot as plt
import json
import re


def main():
    matplotlib.rcParams['legend.fontsize'] = 8

    path = './benchmark.txt'
    with open(path, 'r', encoding='utf8') as f:
        table = json.loads(f.read())
    _table = dict((item['title'], item['data']) for item in table)
    table = {}
    for title, data in _table.items():
        if title.startswith('memtable-btree_'):
            if title.startswith('memtable-btree_builtin'):
                pass
            elif title.startswith('memtable-btree_7'):
                pass
            else:
                continue
        table[title] = data

    # define title rules to match
    rules = [
        (0, 0, r'memtable-(.*?)-rand-read'),
        (0, 1, r'memtable-(.*?)-rand-write'),
        (0, 2, r'memtable-(.*?)-seq-read'),
        (0, 3, r'memtable-(.*?)-seq-write'),
        (1, 0, r'sstable-seq-scan-speed'),
        (1, 1, r'sstable-(seq|rand)-read-speed'),
        (1, 2, r'bloom-(.*?)-perf'),
    ]
    figure, axis = plt.subplots(2, 4)
    for subplotx, subploty, rule in rules:
        pairs = []
        for title, data in table.items():
            if re.match(rule, title):
                pairs.append((title, data))
        subplot = axis[subplotx, subploty]
        for title, data in pairs:
            X = [item['x'] for item in data]
            Y = [item['y'] for item in data]
            label = re.findall(rule, title)
            label = label[0] if len(label) > 0 else '-'
            subplot.plot(X, Y, label=label)
        subplot.set_title(rule, fontsize=10)
        subplot.legend(loc='best')
    plt.show()
    return


if __name__ == '__main__':
    main()
