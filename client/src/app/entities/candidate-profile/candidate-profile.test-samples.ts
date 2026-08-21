import dayjs from 'dayjs/esm';

import { ICandidateProfile, NewCandidateProfile } from './candidate-profile.model';

export const sampleWithRequiredData: ICandidateProfile = {
  id: 22264,
  userId: "à l'exception de de la part de",
};

export const sampleWithPartialData: ICandidateProfile = {
  id: 27061,
  userId: 'adepte soit précisément',
  fullName: 'sans doute pourvu que envers',
  email: 'Armelle_Guerin0@hotmail.fr',
  location: 'mature puisque pour',
  skills: '../fake-data/blob/hipster.txt',
  languages: '../fake-data/blob/hipster.txt',
  education: '../fake-data/blob/hipster.txt',
  certifications: '../fake-data/blob/hipster.txt',
  createdAt: dayjs('2023-12-11T13:14'),
};

export const sampleWithFullData: ICandidateProfile = {
  id: 32302,
  userId: 'insolite sincère adepte',
  fullName: 'descendre éloigner de façon à',
  email: 'Romain65@gmail.com',
  location: 'déplacer épanouir',
  yearsOfExperience: 20157,
  skills: '../fake-data/blob/hipster.txt',
  experiences: '../fake-data/blob/hipster.txt',
  preferredRoles: '../fake-data/blob/hipster.txt',
  languages: '../fake-data/blob/hipster.txt',
  education: '../fake-data/blob/hipster.txt',
  certifications: '../fake-data/blob/hipster.txt',
  rawMarkdown: '../fake-data/blob/hipster.txt',
  cvFilename: 'à même longtemps rectangulaire',
  embeddingModel: 'parce que ensemble',
  embeddedAt: dayjs('2023-12-10T20:36'),
  createdAt: dayjs('2023-12-10T17:18'),
  updatedAt: dayjs('2023-12-11T04:18'),
};

export const sampleWithNewData: NewCandidateProfile = {
  userId: 'perplexe électorat autour de',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
