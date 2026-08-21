import dayjs from 'dayjs/esm';

import { IOfferTailoredResume, NewOfferTailoredResume } from './offer-tailored-resume.model';

export const sampleWithRequiredData: IOfferTailoredResume = {
  id: 31914,
  userId: 'clac bzzz',
  data: '../fake-data/blob/hipster.txt',
};

export const sampleWithPartialData: IOfferTailoredResume = {
  id: 5318,
  userId: 'pendant',
  data: '../fake-data/blob/hipster.txt',
  title: 'aïe',
  createdAt: dayjs('2023-12-11T09:12'),
};

export const sampleWithFullData: IOfferTailoredResume = {
  id: 17496,
  userId: 'sous minuscule',
  data: '../fake-data/blob/hipster.txt',
  title: 'là-haut',
  createdAt: dayjs('2023-12-10T15:43'),
};

export const sampleWithNewData: NewOfferTailoredResume = {
  userId: 'résulter',
  data: '../fake-data/blob/hipster.txt',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
