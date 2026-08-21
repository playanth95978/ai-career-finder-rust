import { IAutoApplyConfig, NewAutoApplyConfig } from './auto-apply-config.model';

export const sampleWithRequiredData: IAutoApplyConfig = {
  id: 23322,
  userId: 'vouh',
};

export const sampleWithPartialData: IAutoApplyConfig = {
  id: 24225,
  userId: 'encore loufoque drelin',
  maxPerDay: 7914,
  sources: '../fake-data/blob/hipster.txt',
};

export const sampleWithFullData: IAutoApplyConfig = {
  id: 17139,
  userId: 'propre tic-tac vite',
  mode: 'AUTO',
  minScore: 24529.78,
  maxPerDay: 30868,
  sources: '../fake-data/blob/hipster.txt',
};

export const sampleWithNewData: NewAutoApplyConfig = {
  userId: 'de la part de à la merci prestataire de services',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
