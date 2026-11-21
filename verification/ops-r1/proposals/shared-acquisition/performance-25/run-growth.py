"""Exact growth cells through the existing source-verifying 60s/512MiB monitor."""
import hashlib,json,os,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
ISO=ROOT/'target/ops-r1/m3-isolated-23'
VERSION=sys.argv[1]
BUILD=Path(sys.argv[2]).resolve()
CASES=sys.argv[3:] or ['flat-scripts-1','flat-scripts-4','flat-scripts-16','directories-scripts-4','directories-scripts-16','fixed-1','fixed-4','fixed-16','flat-modules-16','directories-modules-16']
ADAPTER=ISO/'verification/ops-r1/proposals/shared-acquisition/execution/capture.py'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    build=json.loads(BUILD.read_bytes())
    assert build['launcher_sha256']==sha(ADAPTER)
    out=ROOT/'target/ops-r1'/('performance-25-'+VERSION+'-'+time.strftime('%Y%m%d-%H%M%S'));out.mkdir(exist_ok=False)
    record={'source_version':VERSION,'build_receipt':str(BUILD),'build_receipt_sha256':sha(BUILD),'source_tree_sha256':build['source_tree_sha256'],'executable_sha256':build['executable_sha256'],'selected_cases':CASES,'runs':[],'status':'RUNNING'}
    save=lambda:(out/'review.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
    save()
    for case in CASES:
        parts=case.split('-');fixed=parts[0]=='fixed'
        layout,form=('flat','scripts') if fixed else (parts[0],parts[1])
        n=int(parts[-1]);assert n in (1,4,16,64)
        env=dict(os.environ,OPS_R1_STAGE='green',OPS_R1_GREEN_SOURCE=VERSION,OPS_R1_LAYOUT=layout,OPS_R1_FORM=form,OPS_R1_REQUESTS='1' if fixed else '2')
        print('START '+case,flush=True)
        run=subprocess.run([sys.executable,str(ADAPTER),'--build-receipt',str(BUILD),'--case',case,'--wall-seconds','60','--memory-mib','512','--stop-wait','5'],cwd=ISO,env=env,capture_output=True,text=True)
        (out/(case+'-adapter.txt')).write_text(run.stdout+run.stderr,encoding='utf-8')
        try:
            folder=Path(run.stdout.strip());assert folder.resolve().is_relative_to((ISO/'target/ops-r1').resolve())
            receipt=json.loads((folder/'receipt.json').read_bytes())
            data=json.loads((folder/'performance.json').read_bytes()) if (folder/'performance.json').exists() else None
            entry={'case':case,'layout':layout,'form':form,'nominal_units':n,'receipt':str(folder/'receipt.json'),'receipt_sha256':sha(folder/'receipt.json'),'observations':receipt,'performance':data,'performance_sha256':sha(folder/'performance.json') if data is not None else None}
            record['runs'].append(entry);save()
            print(case+': '+receipt['status']+' '+str(receipt.get('elapsed_seconds')),flush=True)
            if run.returncode or receipt['status']!='TRACKED_TEST_COMPLETED' or not receipt['post_source_and_binary_identity_verified']:
                record['status']='STOPPED_FOR_REVIEW';save();print(str(out),flush=True);return 1
        except Exception as error:
            record.update(status='INFRASTRUCTURE_STOP',error=str(error));save();print(str(out),flush=True);return 1
    record['status']='ALL_SELECTED_CELLS_COMPLETED';save();print(str(out),flush=True);return 0
if __name__=='__main__':sys.exit(main())
