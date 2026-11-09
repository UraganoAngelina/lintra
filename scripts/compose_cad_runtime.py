"""Reproduce the CAD83 candidate from the pinned received kit and reviewed components.

Development tool only: the committed flow/catalogs are sufficient at runtime and
in CI. Never run on patient inputs. This transformation preserves all source
fields, presence expressions and notes; reviewed components supply the approved
generic DSL lowering. Qualification/status are recorded outside the schema so
that a gate cannot change its semantic hash or invalidate approved snapshots.
"""
import copy,json,hashlib,re
from pathlib import Path
repo=Path(__file__).resolve().parents[1]
kit=repo.parent/'LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE'
# Fail before writing if the received sources differ from the documentary audit.
# A changed normative source needs an explicit review, never a silent regeneration.
for trace in ['A', 'B1', 'B2', 'C', 'D']:
 audit=json.loads((repo/f'docs/normative/cad/{trace}-fields.json').read_text())
 for entry in audit['provenance']:
  if entry['role'] in ['received_trace', 'received_manifest', 'received_flow']:
   path=repo.parent/'LINTRA_STARTER_KIT_FINAL_20261016'/entry['kit_path']
   if hashlib.sha256(path.read_bytes()).hexdigest()!=entry['sha256']:
    raise ValueError(f'Received source changed: {entry["kit_path"]}')
source=json.loads((kit/'flow_schema.json').read_text())
components={p.parent.name:json.loads(p.read_text()) for p in (repo/'schemas/cad/components').glob('*/component.json')}
progress=components['2024-06.progressives.1']
flow=copy.deepcopy(json.loads((repo/'tests/golden/synthetic/v1/schema.json').read_text()))
flow.update(schema_format_version=3,flow_id='CAD',schema_version='2024-06.runtime-candidate.1',regulatory_version='2024-06',source={'scope':'83-field authoring candidate, runtime unverified, not FROZEN','full_normative_compliance':False,'legacy_flow_sha256':hashlib.sha256((kit/'flow_schema.json').read_bytes()).hexdigest()},parameters=progress['parameters'],persistence_policy=copy.deepcopy(progress['persistence_policy']),required_capabilities=[],reference_data=components['2024-06.clinical-lookups.1']['reference_data'])
flow['persistence_policy']['temporal_rules']=[]
flow['traces']=[]
ledger=[]
for legacy in source['traces']:
 t=copy.deepcopy(json.loads((repo/'tests/golden/synthetic/v1/schema.json').read_text())['traces'][0]);id=legacy['trace_id'];t.update(id=id,name=legacy['trace_name'],record_length=legacy['record_length'],record_key=legacy['record_key'],fields=[],named_conditions=copy.deepcopy(legacy['named_conditions']),joins=[],derived_values={},rules=[],documented_non_executable_rules=copy.deepcopy(legacy['documented_non_executable_rules']))
 originals={f['id']:f for f in legacy['fields']}
 def lower_expr(e):
  """Encode legacy numeric literals at their exact declared width, recursively."""
  if isinstance(e,list):return [lower_expr(x) for x in e]
  if not isinstance(e,dict):return e
  e={k:lower_expr(v) for k,v in e.items()}
  for op in ['eq','ne','in','not_in']:
   if op in e:
    item=e[op];f=originals[item['ref']['field']]
    if f['physical']['type']=='numeric':
     w=f['physical']['length']
     if 'value' in item:item['value']=item['value'].zfill(w)
     if 'values' in item:item['values']=[x.zfill(w) for x in item['values']]
  return e
 t['named_conditions']=lower_expr(t['named_conditions'])
 for f in legacy['fields']:
  f=copy.deepcopy(f);p=f['physical'];kind=p['type'];w=p['length']
  f['physical']={'type':kind,'start':p['start'],'length':w,'alignment':{'kind':'exact'} if kind!='alphanumeric' else {'kind':'right_pad','fill_byte':32,'content_policy':'strip_trailing_fill'}}
  if kind=='date':f['physical']['format']='DDMMYYYY'
  else:f['physical']['lexical']={'kind':'encoded_text'}
  if f['missing_value']['kind']=='spaces':f['missing_value']={'kind':'spaces'}
  f['presence']=lower_expr(f['presence'])
  cs=[]
  for c in f['constraints']:
   k=c['kind']
   if k=='allowed_values':cs.append({'kind':k,'values':[s.zfill(w) for s in c['values']] if kind=='numeric' else c['values'],'target':'raw' if kind=='numeric' else 'logical'})
   elif k in ['digits_only','digits_only_if_present']:cs.append({'kind':'regex','match':'required','target':'raw','pattern':r'\A[0-9]{'+str(w)+r'}\z'})
   elif k in ['date','date_if_present']:cs.append({'kind':'date','format':'DDMMYYYY','calendar_valid':True})
   elif k in ['reference_data','reference_data_if_present']:cs.append({'kind':'reference_data','datasets':['ICD9' if c['dataset']=='cad-icd9-cm-clinical-signs' else 'PRESTATIONS'],'fallback_values':[],'metadata_bindings':[]})
   elif k=='positional_bitset':
    if 'positions' in c:continue # Replaced below by the already qualified T70 component.
    cs.append({'kind':'regex','match':'required','target':'raw','pattern':r'\A'+''.join('['+''.join(re.escape(x) for x in c['allowed_values'])+']' if i in c['used_positions'] else ' ' for i in range(1,w+1))+r'\z'})
   else:raise ValueError((f['id'],k))
  f['constraints']=cs
  if f['id']=='2.12':f['missing_value']={'kind':'spaces'} # DEC-05-CAD-ICPC-000, explicit ratification.
  t['fields'].append(f)
 # Reuse reviewed field implementations without replacing source presence.
 for name in ['2024-06.calendar-age.1','2024-06.clinical-lookups.1','2024-06.suspensions.1','2024-06.conclusions.1','2024-06.inactivity.1']:
  component=components[name];part=component.get('traces',{}).get(id,{})
  for f in part.get('fields',[]):
   own=next(x for x in t['fields'] if x['id']==f['id']);assert own['physical']['start']==f['physical']['start'] and own['physical']['length']==f['physical']['length'];own['physical']=copy.deepcopy(f['physical']);own['constraints']=copy.deepcopy(f['constraints'])
  for j in part.get('joins',[]):
   if not any(old['id']==j['id'] for old in t['joins']):t['joins'].append(copy.deepcopy(j))
  t['derived_values'].update(copy.deepcopy(part.get('derived_values',{})))
  t['named_conditions'].update(copy.deepcopy(part.get('named_conditions',{})))
  for rule in part.get('rules',[]):
   old=next((old for old in t['rules'] if old['id']==rule['id']),None)
   if old is None:t['rules'].append(copy.deepcopy(rule))
   else:assert old==rule
 if id=='B1':
  t70=components['2024-06.t70.1']
  own=next(f for f in t['fields'] if f['id']=='2.22');copyfield=next(f for f in t70['fields'] if f['id']=='2.22');own.update({k:copy.deepcopy(copyfield[k]) for k in ['physical','constraints']})
 ledger.append({'trace':id,'fields':len(t['fields']),'source_trace_sha256':hashlib.sha256((kit/('trace_'+id.lower()+'.json')).read_bytes()).hexdigest(),'all_original_coordinates_verified':all(f['physical']['start']==originals[f['id']]['physical']['start'] and f['physical']['length']==originals[f['id']]['physical']['length'] for f in t['fields']),'unresolved_non_executable_declarations':t['documented_non_executable_rules']})
 flow['traces'].append(t)
for name in ['2024-06.suspensions.1','2024-06.conclusions.1','2024-06.inactivity.1']:
 flow['persistence_policy']['temporal_rules']+=copy.deepcopy(components[name]['persistence_policy']['temporal_rules'])
for name in ['2024-06.progressives.1','2024-06.calendar-age.1','2024-06.suspensions.1','2024-06.conclusions.1','2024-06.inactivity.1']:
 for cap in components[name]['required_capabilities']:
  if cap not in flow['required_capabilities']:flow['required_capabilities'].append(cap)
assert sum(t['fields'].__len__() for t in flow['traces'])==83
# Complete the explicit source relationships, keeping normative PKs intact.
flow['required_capabilities'].append('history.existence.v1')
missing=lambda code,msg: {'policy':'unknown','diagnostic':{'code':code,'message_template':msg}}
violation=lambda code,msg: {'policy':'violation','error_code':code,'diagnostic':{'code':code,'message_template':msg}}
def relation(id,target,src,dst,card='zero_or_many',consensus=None):
 """Declare namespace-scoped evidence; unavailable input is explicitly Unknown."""
 j={'id':id,'target_trace':target,'source_fields':src,'target_fields':dst,'cardinality':card,'scope':'current_submission','namespace_parameters':['sending_ulss'],
 'on_missing_record':violation('E_CAD_RELATED_RECORD_MISSING','Required related record is absent.'),
 'on_missing_trace':missing('W_CAD_RELATED_TRACE_UNAVAILABLE','Required input is unavailable.')}
 if consensus is not None:j['reduction']={'kind':'consensus','on_conflict':'unknown',**consensus}
 return j
def rule(id,when,assertions,code,message):return {'id':id,'when':when,'assertions':assertions,'error_code':code,'message_template':message}
by={t['id']:t for t in flow['traces']}
for trace,event in [('B1','2.1'),('B2','3.1'),('C','4.1'),('D','5.1')]:
 j=relation(f'{trace}_TO_A_EVENT','A',[event],['1.23'],'exactly_one');by[trace]['joins'].append(j)
 by[trace]['rules'].append(rule(f'CAD-{trace}-PARENT-EVENT',{'const':True},[{'kind':'record_exists','join':j['id']}],'E_CAD_EVENT_PARENT_MISSING','Record has no parent event in its namespace.'))
j=relation('D_TO_C_ACCESS','C',['5.1','5.2','5.3','5.4'],['4.1','4.2','4.4','4.5']);by['D']['joins'].append(j)
by['D']['rules'].append(rule('CAD-PROVISION-ACCESS',{'const':True},[{'kind':'set_exists','join':j['id'],'predicate':{'const':True}}],'E_CAD_MISSING_ACCESS_FOR_PROVISION','Provision has no related access.'))
j=relation('B1_INITIAL_HISTORY','B1',['2.1'],['2.1']);j['scope']='history_including_current';by['B1']['joins'].append(j)
by['B1']['rules'].append(rule('CAD-INITIAL-HISTORY',{'eq':{'ref':{'field':'2.8'},'value':'2'}},[{'kind':'set_exists','join':j['id'],'predicate':{'eq':{'ref':{'field':'2.8'},'value':'1'}}}],'E_CAD_INITIAL_EVALUATION_REQUIRED','A reevaluation requires a valid initial evaluation in approved history or current input.'))
# The professional-code rule is a declarative comparison with a qualified
# consensus start, never a first B1 row or an engine branch on a clinical ID.
for trace,professional,event in [('C','4.5','4.1'),('D','5.4','5.1')]:
 j=relation(f'{trace}_TO_B1_START','B1',[event],['2.1'],consensus={'fields':['2.4'],'include_if':{'const':True},'diagnostic':{'code':'W_CAD_CARE_START_AMBIGUOUS','message_template':'Care start dates disagree.'}})
 j['on_missing_record']=missing('W_CAD_CARE_START_UNAVAILABLE','Care start is unavailable.');by[trace]['joins'].append(j)
 pattern=r'\A[0-9]{4}(?:202[5-9]|20[3-9][0-9]|2[1-9][0-9]{2}|[3-9][0-9]{3})\z'
 by[trace]['rules'].append(rule(f'CAD-{professional}-15',{'eq':{'ref':{'field':professional},'value':'15'}},[{'kind':'constraint','field':{'trace':'B1','field':'2.4','join':j['id']},'constraint':{'kind':'regex','match':'forbidden','target':'raw','pattern':pattern}}],'E_CAD_PROFESSIONAL_15_AFTER_2024','Professional code 15 is forbidden for care episodes starting in 2025 or later.'))
resolved={
 'CAD-AGE-UNDER18':['A.rules.CAD-AGE-GUARDIAN'],
 'CAD-AGE-3-6':['B1.rules.CAD-AGE-MOBILITY','B1.rules.CAD-AGE-FUNCTION'],
 'CAD-17':['persistence_policy.temporal_rules.inactivity_with_pauses'],
 'CAD-14':['persistence_policy.temporal_rules.interval_exclusion','persistence_policy.temporal_rules.after_date_restriction'],
 'CAD-4.5-15':['C.rules.CAD-4.5-15'], 'CAD-5.4-15':['D.rules.CAD-5.4-15'],
 'CAD-4.9-SPECIALIST':['C.fields.4.9.presence.forbidden_if'],
 'CAD-4.10-OSS':['C.fields.4.10.presence.forbidden_if'],
}
exclusions={
 'CAD-IDENTIFIER-CROSSCHECK':'CAD-03 / docs/normative/cad/rules-and-joins.md: no semantic 1.2/1.3 or fiscal checksum check by author decision',
 'CAD-2.11-12-ICPC':'DEC-05-CAD-ICPC-000 / TASK_PROMPTS/T-CAD-11.md: no additional ICPC lookup, mandatory values and 000 handling preserved',
}
resolutions=[]
for trace in flow['traces']:
 deferred=[]
 for note in trace['documented_non_executable_rules']:
  n=copy.deepcopy(note)
  if n['id'] in resolved:
   resolutions.append({'source_trace':trace['id'],'source_note':n,'status':'IMPLEMENTED_APPROVED_CONTROL','executors':resolved[n['id']]})
  else:
   assert n['id'] in exclusions,n
   n['policy']='not_enforced_in_this_version';n['decision_reference']=exclusions[n['id']];deferred.append(n)
   resolutions.append({'source_trace':trace['id'],'source_note':note,'status':'EXCLUDED_BY_EXISTING_AUTHOR_DECISION','decision_reference':exclusions[n['id']]})
 trace['documented_non_executable_rules']=deferred
flow['source'].update(scope='Complete 83-field CAD runtime candidate; qualification recorded separately, not FROZEN',control_resolutions=resolutions,
 reference_scope='Only the two reviewed ICD9/prestations catalogs declared by CAD11. ICPC/ISTAT excluded by decision; no unprovided specialist catalog invented.')
out=repo/'schemas/operational/CAD/2024-06.runtime-candidate.1';(out/'data').mkdir(parents=True,exist_ok=True)
for sourcefile in (repo/'schemas/cad/components/2024-06.clinical-lookups.1/data').glob('*.json'):(out/'data'/sourcefile.name).write_bytes(sourcefile.read_bytes())
(out/'flow.json').write_text(json.dumps(flow,indent=2)+'\n')
print('Composed 83 source fields, 5 traces; mandatory notes mapped to executors, original exclusions traceable; not runtime-qualified yet')
