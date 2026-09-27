"""Final paging host receipts. Child-only preference isolation, no old evidence overwrite."""
import hashlib,json,os,pathlib,subprocess,time,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]

def run(exe,out,root):
    source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
    sha=hashlib.sha256(exe.read_bytes()).hexdigest()
    config=out/'terrain-local-appdata'
    preference=config/'Merely/Isometry/terrain.json'
    preference.parent.mkdir(parents=True,exist_ok=False)
    preference.write_text(json.dumps(dict(version=1,atlas_budget_mib=1)))
    (out/'terrain-seeded-preference.json').write_bytes(preference.read_bytes())
    cases=[('terrain-change',dict(ISOMETRY_SYNTH='256',ISOMETRY_TERRAIN_SELFTEST='change'),config),
           ('terrain-restart',dict(ISOMETRY_SYNTH='256',ISOMETRY_TERRAIN_SELFTEST='verify'),config),
           ('headed-demo',dict(ISOMETRY_OVERLAY_SELFTEST='1'),None),
           ('headed-256',dict(ISOMETRY_SYNTH='256',ISOMETRY_OVERLAY_SELFTEST='1'),None),
           ('headroom-zero',dict(ISOMETRY_SYNTH='256',ISOMETRY_OVERLAY_SELFTEST='1',ISOMETRY_SCENE_HEADROOM='0'),None),
           ('overlay-control',dict(ISOMETRY_SYNTH='256'),None)]
    for name,switches,local in cases:
        directory=out/name
        directory.mkdir(exist_ok=False)
        env={key:value for key,value in os.environ.items() if not key.startswith('ISOMETRY_')}
        env.update(LOCALAPPDATA=str(local or directory/'local-appdata'),ISOMETRY_SCENE_BOARD='1',ISOMETRY_PROFILE='1',ISOMETRY_CAPTURE_DIR=str(directory),**switches)
        (directory/'source.json').write_text(json.dumps(dict(source=source,binary=str(exe),sha256=sha,build='release',cwd=str(root),env={k:v for k,v in env.items() if k.startswith('ISOMETRY_') or k=='LOCALAPPDATA'}),indent=2))
        print(name,flush=True)
        with (directory/'stdout.log').open('xb') as stdout,(directory/'stderr.log').open('xb') as stderr:
            process=subprocess.Popen([str(exe)],cwd=root,env=env,stdout=stdout,stderr=stderr)
            deadline=time.monotonic()+30
            while time.monotonic()<deadline and process.poll() is None:
                if (directory/'isometry_capture.png').exists():
                    time.sleep(1)
                    break
                time.sleep(.25)
            live=process.poll() is None
            if live: process.kill()
            code=process.wait()
        text=(directory/'stderr.log').read_text(errors='replace')
        assert live, (name,'host exited before bounded stop',code,text[-2000:])
        assert (directory/'isometry_capture.png').exists(),(name,'capture missing')
        assert 'panicked at' not in text and 'scene board: ' not in text,(name,'host error')
        if name.startswith('terrain-'):
            assert '[terrain-host-receipt] PASS' in text,(name,'host assertion marker absent')
            saved=json.loads(preference.read_text())
            assert saved==dict(version=1,atlas_budget_mib=2),saved
            (directory/'saved-preference.json').write_bytes(preference.read_bytes())
        (directory/'termination.json').write_text(json.dumps(dict(exit=code,stopped_after_capture=live)))
    from PIL import Image
    import numpy as np
    def pixels(name):return np.asarray(Image.open(out/name/'isometry_capture.png').convert('RGBA'))
    first=pixels('headed-256');same=pixels('headroom-zero');control=pixels('overlay-control')
    assert first.shape==same.shape==control.shape
    drift=int(np.any(first!=same,axis=2).sum());positive=int(np.any(first!=control,axis=2).sum())
    assert drift==0,('headroom drift',drift)
    assert positive>0,('overlay positive control absent',positive)
    (out/'headed-comparison.json').write_text(json.dumps(dict(headroom_drift=drift,overlay_control_differing_pixels=positive,shape=first.shape),indent=2))

if __name__=='__main__':
    run(pathlib.Path(sys.argv[1]).resolve(),pathlib.Path(sys.argv[2]).resolve(),ROOT)
