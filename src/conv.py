with open("sd","r") as f:
    for line in f.readlines():
        line = line.strip()
        line = line.replace("\"","\\\"")
        line = line.replace("{","{{")
        line = line.replace("}","}}")
        print("writeln!(&mut w, \"" + line + "\").unwrap();")