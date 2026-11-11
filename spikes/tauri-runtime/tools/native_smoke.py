"""Foreground native smoke; only synthetic runtime data, no desktop capture."""
import hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
exe=Path(sys.argv[1]).resolve(strict=True)
print("binary_sha256",hashlib.sha256(exe.read_bytes()).hexdigest(),flush=True)
with tempfile.TemporaryDirectory(prefix="lintra-native-spike-",dir="/tmp") as rt:
    env=os.environ.copy()
    for variable,dirname in [("XDG_DATA_HOME","data"),("XDG_CACHE_HOME","cache"),("XDG_CONFIG_HOME","config")]:
        dest=Path(rt)/dirname;dest.mkdir(mode=0o700);env[variable]=str(dest)
    env["GDK_BACKEND"]="wayland"
    result=subprocess.run([str(exe),"--smoke-test"],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=45)
    print(result.stdout,flush=True)
    print("native_process_exit",result.returncode,flush=True)
    assert result.returncode==0,"native process did not exit successfully"
    prefix="LINTRA_RUNTIME_RENDER_ACK "
    acks=[json.loads(line[len(prefix):]) for line in result.stdout.splitlines() if line.startswith(prefix)]
    assert len(acks)==1,"exactly one native React/Rust render acknowledgment required"
    assert acks[0]["protocol"]==1 and acks[0]["width"]>=640 and acks[0]["height"]>=480,acks
    print("NATIVE_WAYLAND_SMOKE_PASS",json.dumps(acks[0]),flush=True)
