import dayjs from 'dayjs/esm';

import { ICvResume, NewCvResume } from './cv-resume.model';

export const sampleWithRequiredData: ICvResume = {
  id: 5145,
  userId: 'vroum',
  data: '../fake-data/blob/hipster.txt',
  versionNumber: 25534,
};

export const sampleWithPartialData: ICvResume = {
  id: 3059,
  userId: 'placer joliment avant de',
  title: 'fade davantage',
  data: '../fake-data/blob/hipster.txt',
  versionNumber: 29772,
};

export const sampleWithFullData: ICvResume = {
  id: 23758,
  userId: 'dessiner près de ouf',
  title: 'population du Québec super',
  template: 'administration',
  data: '../fake-data/blob/hipster.txt',
  versionNumber: 28578,
  createdAt: dayjs('2023-12-11T11:23'),
  updatedAt: dayjs('2023-12-11T03:49'),
};

export const sampleWithNewData: NewCvResume = {
  userId: 'même si',
  data: '../fake-data/blob/hipster.txt',
  versionNumber: 3424,
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
