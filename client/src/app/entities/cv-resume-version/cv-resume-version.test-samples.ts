import dayjs from 'dayjs/esm';

import { ICvResumeVersion, NewCvResumeVersion } from './cv-resume-version.model';

export const sampleWithRequiredData: ICvResumeVersion = {
  id: 9405,
  versionNumber: 11041,
  data: '../fake-data/blob/hipster.txt',
};

export const sampleWithPartialData: ICvResumeVersion = {
  id: 31220,
  versionNumber: 5780,
  title: 'avant que',
  template: 'cerner chez',
  data: '../fake-data/blob/hipster.txt',
};

export const sampleWithFullData: ICvResumeVersion = {
  id: 8668,
  versionNumber: 15778,
  title: 'alors que bousculer',
  template: 'grrr',
  data: '../fake-data/blob/hipster.txt',
  createdAt: dayjs('2023-12-10T21:28'),
};

export const sampleWithNewData: NewCvResumeVersion = {
  versionNumber: 527,
  data: '../fake-data/blob/hipster.txt',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
