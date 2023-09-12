#!/usr/bin/python3

# ldbdump - dumps LevelDB keys/values
#
# a LevelDB is a dir with files such a these:
# 000050.ldb  000100.log  CURRENT  LOCK  LOG  MANIFEST-000099
#
# sources: https://github.com/tos-kamiya/levelobjdb dump()
# sources: https://gist.githubusercontent.com/mkorthof/412e3cb64785c4f136bbb7f6a9d3a71c/raw/e407eda812448e711a1a496cfd480c227eebacfb/ldbdump.py

import os
import leveldb
import json
import sys

# output file:
filename = "ldb_out.json"

home_dir = os.path.expanduser("~")
repair, pk, pkv, p8kv, pj, p8j = False, False, False, False, False, False

# dbdir: leveldb database
if (len(sys.argv) > 1):
  if not (sys.argv[1].startswith('-')):
    dbdir = sys.argv[1]
  if (len(sys.argv) > 2):
    if ("-r" in sys.argv):
      repair = True
    if ("-k" in sys.argv):
      pk = True
    if ("-kv" in sys.argv):
      pkv = True
    if ("-8kv" in sys.argv):
      p8kv = True
    if ("-j" in sys.argv):
      pj = True
    if ("-8j" in sys.argv):
      p8j = True
    if not (sys.argv[2].startswith('-')):
      dbdir = sys.argv[2]
else:
  print ('\n' + sys.argv[0].split('/')[-1] + ' -[j|k|kv|8j|8kv] <dbdir>')
  print ('output leveldb database in <dbdir> to ' + filename + '\n')
  print ('USAGE:')
  print (' -r   repair db first')
  print (' -k   print keys')
  print (' -kv  print keys and values')
  print (' -j   print json\n')
  print (' -8kv print keys and values (utf8)')
  print (' -8j  print full valid json (utf8)\n')
  exit()

# Override dbdir:
# dbdir = home_dir + "/tmp/pkehgijcmpdhfbdbbnkijodmdjhbjlgp"
# dbdir = home_dir + "/tmp/gcknhkkoolaabfmlnjonogaaifnjlfnp"

print ('')

try:
  dbdir
except:
  print ('ERROR: dbdir not specified\n')
  exit(1)

if not os.path.isdir(dbdir):
  print ('ERROR: dbdir "' + dbdir + '" doesnt exist\n')
  exit(1)

# Repair db in dbdir, eg:
# Corruption: corrupted compressed block contents
if (repair):
  print ('Repairing db...\n')
  leveldb.RepairDB('dbdir')

db = leveldb.LevelDB(dbdir)

# Just print keys:
if (pk):
  print ('Printing keys:\n')
  for k in db.RangeIter(include_value = False):
    print ('Key:', k)
  exit()

# Print both keys and values:
if (pkv):
  print ('Printing keys and values:\n')
  for k in db.RangeIter(include_value = False):
    print ('Key:', k, 'Value:', db.Get(k), '\n')
  exit()

# Print json:
if (pj):
  print ('Printing JSON:\n')
  for k in db.RangeIter(include_value = False):
    print (json.loads(db.Get(k).decode('utf-8')), '\n')
  exit()

# Print keys and values:
if (p8kv):
  print ('Printing keys and values(2):\n')
  for k, v in db.RangeIter():
    print ('Key:', k, 'Value:', v)
  exit()

# Try to print full valid json output:
if (p8j):
  print ("{")
  c = 0
  for k, v in db.RangeIter():
    if c > 0:
      print (", ",end='')
    print (json.dumps(k.decode('utf-8'), indent=0),end='')
    print (": ",end='')
    print (json.dumps(json.loads(v.decode('utf-8')),indent=4),end='')
    c += 1
  if c >= 1:
    print ('')
  print ("}\n")
  exit()

# Function dump() does the same but outputs to filename:
def dump(filename, indent=4):
  """
  writes key-value pairs to `out`.
  specify an optional parameter `indent` to change indent step.
  """
  out = open(filename, 'w')
  out.write("{\n")
  c = 0
  for k, v in db.RangeIter():
    if c > 0:
      out.write(", \n")
    out.write(json.dumps(k.decode('utf-8'), indent=0))
    out.write(": ")
    out.write(json.dumps(json.loads(v.decode('utf-8')), indent=indent))
    c += 1
  if c >= 1:
      out.write("\n")
  out.write("}\n")
  out.close
  print ("Wrote JSON output from db in dir \"" + dbdir + "\" to: " + filename + "\n")

dump(filename)
#dump("ldb_out.json")
#dump("badger_ldb_out2.json")

