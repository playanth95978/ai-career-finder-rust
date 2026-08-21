import dayjs from 'dayjs/esm';

import { IJobApplication, NewJobApplication } from './job-application.model';

export const sampleWithRequiredData: IJobApplication = {
  id: 3460,
  userId: 'pendre plouf bzzz',
};

export const sampleWithPartialData: IJobApplication = {
  id: 27723,
  userId: 'malgré',
  status: 'GHOSTED',
  notes: '../fake-data/blob/hipster.txt',
  matchScore: 28711.55,
  appliedAt: dayjs('2023-12-11T13:21'),
};

export const sampleWithFullData: IJobApplication = {
  id: 1949,
  userId: "construire d'avec",
  status: 'WITHDRAWN',
  coverLetter: '../fake-data/blob/hipster.txt',
  notes: '../fake-data/blob/hipster.txt',
  matchScore: 28249.04,
  createdAt: dayjs('2023-12-10T19:13'),
  updatedAt: dayjs('2023-12-10T18:27'),
  appliedAt: dayjs('2023-12-11T08:28'),
};

export const sampleWithNewData: NewJobApplication = {
  userId: 'à côté de',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
