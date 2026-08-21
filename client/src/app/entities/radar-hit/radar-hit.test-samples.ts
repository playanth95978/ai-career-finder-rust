import dayjs from 'dayjs/esm';

import { IRadarHit, NewRadarHit } from './radar-hit.model';

export const sampleWithRequiredData: IRadarHit = {
  id: 13360,
  userId: 'séculaire chier au défaut de',
};

export const sampleWithPartialData: IRadarHit = {
  id: 21177,
  userId: 'gigantesque ouille parce que',
  whyYou: '../fake-data/blob/hipster.txt',
  seen: false,
  dismissed: true,
  createdAt: dayjs('2023-12-11T00:11'),
};

export const sampleWithFullData: IRadarHit = {
  id: 22289,
  userId: 'conseil d’administration jamais lunatique',
  score: 4227.95,
  whyYou: '../fake-data/blob/hipster.txt',
  seen: false,
  dismissed: true,
  createdAt: dayjs('2023-12-11T02:19'),
};

export const sampleWithNewData: NewRadarHit = {
  userId: 'à raison de refroidir attacher',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
