import dayjs from 'dayjs/esm';

import { IJobOffer, NewJobOffer } from './job-offer.model';

export const sampleWithRequiredData: IJobOffer = {
  id: 4571,
  title: 'discerner',
};

export const sampleWithPartialData: IJobOffer = {
  id: 23391,
  title: 'sonner membre à vie',
  company: 'par rapport à',
  location: 'résulter tenir meubler',
  description: '../fake-data/blob/hipster.txt',
  skills: '../fake-data/blob/hipster.txt',
  metadata: '../fake-data/blob/hipster.txt',
  rawPayload: '../fake-data/blob/hipster.txt',
  contentHash: 'de sorte que',
  embeddingModel: 'commis de cuisine prestataire de services en guise de',
  indexingError: 'quoique nonobstant',
  source: 'GREENHOUSE',
  sourceId: 'égoïste',
  salaryMin: 13257,
  salaryMax: 12057,
  salaryCurrency: 'jusqu’à ce que au cas où',
  contractType: 'restituer du fait que brave',
  category: 'vorace lasser quelque',
  publishedAt: dayjs('2023-12-10T18:22'),
  createdAt: dayjs('2023-12-11T10:33'),
  expiresAt: dayjs('2023-12-10T15:52'),
};

export const sampleWithFullData: IJobOffer = {
  id: 10912,
  title: 'de façon que présidence',
  company: 'considérer',
  location: 'insipide résoudre',
  country: 'Hongrie',
  remote: true,
  description: '../fake-data/blob/hipster.txt',
  searchText: '../fake-data/blob/hipster.txt',
  skills: '../fake-data/blob/hipster.txt',
  metadata: '../fake-data/blob/hipster.txt',
  rawPayload: '../fake-data/blob/hipster.txt',
  contentHash: 'en outre de',
  embeddingStatus: 'PROCESSING',
  embeddingModel: 'ouch miaou propre',
  reindexVersion: 26365,
  retryCount: 14051,
  indexingError: "d'après",
  source: 'HELLOWORK',
  sourceId: 'extatique',
  applyUrl: 'contre habile',
  salaryMin: 18064,
  salaryMax: 4425,
  salaryCurrency: 'hier laisser',
  contractType: 'marron',
  experienceLevel: 'asseoir jusqu’à ce que regagner',
  category: 'dehors orange patientèle',
  sourceCategory: 'pendant drôlement',
  publishedAt: dayjs('2023-12-10T19:49'),
  createdAt: dayjs('2023-12-11T01:07'),
  indexedAt: dayjs('2023-12-10T23:09'),
  updatedAt: dayjs('2023-12-11T06:18'),
  expiresAt: dayjs('2023-12-10T18:56'),
  lastCheckedAt: dayjs('2023-12-10T23:14'),
};

export const sampleWithNewData: NewJobOffer = {
  title: 'bof hebdomadaire hi',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
