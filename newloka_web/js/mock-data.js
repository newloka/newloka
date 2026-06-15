
/**
 * New Loka Mock Data Generator
 * Seeds IndexedDB with realistic FHIR R4 patients and clinical data.
 */

import * as api from './api.js';

const FAMILIES = ['Smith','Patel','Johnson','Kim','Chen','Singh','Brown','Garcia','Li','Mueller','Dubey','Yadav','Rao','Nair','Shah','Reddy','Mehta','Iyer','Das','Banerjee','Gupta','Khan','Ali','Rahman','Hassan'];
const MALE_NAMES = ['James','Rajesh','Robert','Min-Jun','Wei','Amit','William','Carlos','Xiao','Klaus','Vikram','Ravi','Arjun','Kiran','Raj','Suresh','Rohit'];
const FEMALE_NAMES = ['Maria','Priya','Jennifer','Soo-Min','Li','Sunita','Emily','Isabella','Yan','Hannah','Anjali','Pooja','Neha','Sneha','Meera','Lakshmi','Saraswati'];
const CITIES = ['Mumbai','Delhi','Bangalore','Chennai','Kolkata','Hyderabad','Pune','Ahmedabad','Jaipur','Lucknow','Kanpur','Nagpur','Indore','Thane','Bhopal'];
const CONDITIONS = [
  ['Type 2 Diabetes Mellitus','E11.9'],['Essential Hypertension','I10'],['Asthma','J45.9'],['Acute Bronchitis','J20.9'],
  ['Coronary Artery Disease','I25.10'],['Chronic Kidney Disease','N18.9'],['Osteoarthritis','M19.90'],
  ['Major Depressive Disorder','F32.9'],['Migraine','G43.909'],['Appendicitis','K35.80'],['Pneumonia','J18.9'],
  ['Urinary Tract Infection','N39.0'],['Anemia','D64.9'],['Hypothyroidism','E03.9'],['Hyperlipidemia','E78.5'],
  ['Gastroesophageal Reflux Disease','K21.9'],['COPD','J44.9'],['Atrial Fibrillation','I48.91'],
  ['Heart Failure','I50.9'],['Obesity','E66.9']
];
const MEDS = [
  ['Metformin 500mg','1 tablet twice daily after meals'],['Amlodipine 5mg','1 tablet daily morning'],
  ['Atorvastatin 10mg','1 tablet at bedtime'],['Salbutamol inhaler','2 puffs as needed for wheeze'],
  ['Aspirin 75mg','1 tablet daily after breakfast'],['Omeprazole 20mg','1 tablet before breakfast'],
  ['Levothyroxine 50mcg','1 tablet empty stomach morning'],['Paracetamol 500mg','1-2 tablets every 6 hours for fever'],
  ['Amoxicillin 500mg','1 capsule three times daily for 5 days'],['Insulin Glargine','10 units subcutaneously at bedtime'],
  ['Losartan 50mg','1 tablet daily morning'],['Metoprolol 25mg','1 tablet twice daily'],
  ['Furosemide 40mg','1 tablet daily morning'],['Dextromethorphan','10ml three times daily for cough'],
  ['Cetirizine 10mg','1 tablet at night for allergies']
];
const PROCEDURES = [
  ['Appendectomy','completed','2023-06-15'],['Coronary Angiography','completed','2023-08-20'],
  ['Cataract Surgery','completed','2023-09-10'],['Knee Arthroscopy','completed','2023-07-05'],
  ['Colonoscopy','completed','2023-10-12'],['Tonsillectomy','completed','2023-05-18'],
  ['Laparoscopic Cholecystectomy','completed','2023-11-02'],['Hernia Repair','completed','2023-04-22'],
  ['Dialysis','completed','2023-12-01'],['Pacer Placement','completed','2023-03-14'],
  ['CABG','completed','2023-02-28'],['Bone Marrow Biopsy','completed','2023-01-15'],
  ['ERCP','completed','2023-06-30'],['Thyroidectomy','completed','2023-07-18'],
  ['Craniotomy','completed','2023-08-05'],['Chemotherapy Cycle 1','completed','2023-09-20'],
  ['Radiotherapy Session 1','completed','2023-10-05'],['Liver Biopsy','completed','2023-11-15'],
  ['Spinal Fusion','completed','2023-12-10'],['Skin Graft','completed','2023-12-25']
];
const OBS_TYPES = [
  ['Blood Pressure','mmHg','vital-signs',[[120,80],[130,85],[140,90],[110,70],[125,78]]],
  ['Body Temperature','Celsius','vital-signs',[36.5,37.0,37.5,38.0,36.8]],
  ['Heart Rate','beats/min','vital-signs',[72,68,75,80,65,88]],
  ['Respiratory Rate','breaths/min','vital-signs',[16,18,14,20,12]],
  ['Oxygen Saturation','%','vital-signs',[98,97,96,99,95]],
  ['Body Weight','kg','vital-signs',[65,70,55,80,60,75]],
  ['Height','cm','vital-signs',[165,170,160,175,155]],
  ['BMI','kg/m2','vital-signs',[22,24,20,26,28,21]],
  ['Blood Glucose','mg/dL','laboratory',[90,110,85,120,100,130]],
  ['HbA1c','%','laboratory',[5.5,6.0,7.0,6.5,8.0,5.8]],
  ['Total Cholesterol','mg/dL','laboratory',[180,200,170,220,190]],
  ['HDL Cholesterol','mg/dL','laboratory',[45,50,40,55,35]],
  ['LDL Cholesterol','mg/dL','laboratory',[110,130,90,140,100]],
  ['Triglycerides','mg/dL','laboratory',[150,180,120,200,140]],
  ['Serum Creatinine','mg/dL','laboratory',[0.9,1.1,0.8,1.3,1.0]],
  ['eGFR','mL/min/1.73m2','laboratory',[90,80,100,70,110]],
  ['Hemoglobin','g/dL','laboratory',[13,12,14,11,15]],
  ['WBC Count','cells/uL','laboratory',[7000,6000,8000,5000,9000]],
  ['Platelet Count','cells/uL','laboratory',[250000,200000,300000,180000,220000]],
  ['Sodium','mmol/L','laboratory',[140,138,142,136,144]],
  ['Potassium','mmol/L','laboratory',[4.0,4.5,3.8,5.0,3.5]],
  ['TSH','mIU/L','laboratory',[2.0,3.0,1.5,4.0,2.5]],
  ['Free T4','pmol/L','laboratory',[15,18,12,20,14]],
  ['C-Reactive Protein','mg/L','laboratory',[2,5,1,8,3]],
  ['ESR','mm/hr','laboratory',[15,25,10,30,20]],
  ['Chest X-Ray','','imaging',[]],
  ['ECG','','imaging',[]],
  ['Echocardiography','','imaging',[]],
  ['Ultrasound Abdomen','','imaging',[]],
  ['CT Brain','','imaging',[]],
  ['MRI Spine','','imaging',[]]
];

function pick(arr) { return arr[Math.floor(Math.random()*arr.length)]; }
function randInt(min,max) { return Math.floor(Math.random()*(max-min+1))+min; }
function pad2(n) { return n<10?'0'+n:n; }
function randDate(startYear, endYear) {
  const y = randInt(startYear, endYear);
  const m = randInt(1,12);
  const d = randInt(1,28);
  return `${y}-${pad2(m)}-${pad2(d)}`;
}
function randDateTime(startYear, endYear) {
  const d = randDate(startYear, endYear);
  const h = randInt(8,18);
  const min = randInt(0,59);
  return `${d}T${pad2(h)}:${pad2(min)}:00Z`;
}

export async function seedLocal(count = 15) {
  for (let i=0; i<count; i++) {
    const isMale = Math.random() < 0.5;
    const family = pick(FAMILIES);
    const given = isMale ? pick(MALE_NAMES) : pick(FEMALE_NAMES);
    const city = pick(CITIES);
    const dob = randDate(1950, 2018);
    const pid = crypto.randomUUID ? crypto.randomUUID() : `pat-${Date.now()}-${i}`;

    const patient = {
      resourceType: 'Patient', id: pid,
      identifier: [{system:'http://newloka.org/mrn',value:`MRN${randInt(100000,999999)}`}],
      name: [{family, given:[given]}],
      gender: isMale ? 'male' : 'female',
      birthDate: dob,
      telecom: [{system:'phone',value:`+91-${randInt(7000000000,9999999999)}`}],
      address: [{city, country:'India'}]
    };
    await api.dbPut('patients', patient);

    const encCount = randInt(1,4);
    for (let j=0; j<encCount; j++) {
      const eid = crypto.randomUUID ? crypto.randomUUID() : `enc-${Date.now()}-${j}`;
      const encDate = randDateTime(2023,2025);
      const encounter = {
        resourceType: 'Encounter', id: eid,
        status: pick(['finished','in-progress','planned']),
        class: {code: pick(['AMB','IMP','EMER']), display:'Encounter'},
        subject: {reference: `Patient/${pid}`},
        period: {start: encDate},
        location: [{location:{display: city}}]
      };
      await api.dbPut('encounters', encounter);
    }

    const obsCount = randInt(2,7);
    for (let j=0; j<obsCount; j++) {
      const [text, unit, cat, vals] = pick(OBS_TYPES);
      const obsId = crypto.randomUUID ? crypto.randomUUID() : `obs-${Date.now()}-${j}`;
      let payload = {
        resourceType: 'Observation', id: obsId,
        status: 'final',
        category: [{coding:[{system:'http://terminology.hl7.org/CodeSystem/observation-category',code:cat}]}],
        code: {text},
        subject: {reference: `Patient/${pid}`},
        effectiveDateTime: randDateTime(2023,2025)
      };
      if (text === 'Blood Pressure') {
        const [sys,dia] = pick(vals);
        payload.component = [
          {code:{text:'Systolic'}, valueQuantity:{value:sys, unit:'mmHg'}},
          {code:{text:'Diastolic'}, valueQuantity:{value:dia, unit:'mmHg'}}
        ];
      } else if (unit) {
        const v = pick(vals);
        payload.valueQuantity = {value: v, unit};
      } else {
        payload.valueString = 'Normal study';
      }
      await api.dbPut('observations', payload);
    }

    const condCount = randInt(0,3);
    for (let j=0; j<condCount; j++) {
      const [text,code] = pick(CONDITIONS);
      const cid = crypto.randomUUID ? crypto.randomUUID() : `con-${Date.now()}-${j}`;
      const cond = {
        resourceType: 'Condition', id: cid,
        verificationStatus: {coding:[{code:'confirmed'}]},
        code: {text, coding:[{system:'http://hl7.org/fhir/sid/icd-10',code}]},
        subject: {reference: `Patient/${pid}`},
        onsetDateTime: randDate(2015,2024),
        severity: {text: pick(['mild','moderate','severe'])}
      };
      await api.dbPut('conditions', cond);
    }

    const medCount = randInt(0,4);
    for (let j=0; j<medCount; j++) {
      const [text,instr] = pick(MEDS);
      const mid = crypto.randomUUID ? crypto.randomUUID() : `med-${Date.now()}-${j}`;
      const med = {
        resourceType: 'MedicationRequest', id: mid,
        status: 'active', intent: 'order',
        medicationCodeableConcept: {text},
        subject: {reference: `Patient/${pid}`},
        dosageInstruction: [{text: instr}],
        authoredOn: randDate(2022,2025)
      };
      await api.dbPut('medicationRequests', med);
    }

    const procCount = randInt(0,2);
    for (let j=0; j<procCount; j++) {
      const [text,stat,date] = pick(PROCEDURES);
      const prid = crypto.randomUUID ? crypto.randomUUID() : `proc-${Date.now()}-${j}`;
      const proc = {
        resourceType: 'Procedure', id: prid,
        status: stat,
        code: {text},
        subject: {reference: `Patient/${pid}`},
        performedDateTime: date
      };
      await api.dbPut('procedures', proc);
    }
  }

  // Seed audit
  await api.dbPut('audit', {
    resourceType: 'AuditEvent',
    id: crypto.randomUUID ? crypto.randomUUID() : `aud-${Date.now()}`,
    type: {coding:[{code:'rest'}]},
    action: 'C',
    recorded: new Date().toISOString(),
    outcome: 'Success',
    agent: [{who:{display:'system'}, requestor: true}],
    source: {observer:{display:'browser-node'}},
    entity: [{what:{reference:'Database'}, name:'Local mock data seed'}]
  });

  return { seeded: count };
}
