import { IUserPreference, NewUserPreference } from './user-preference.model';

export const sampleWithRequiredData: IUserPreference = {
  id: 22391,
  userId: 'avant que si',
};

export const sampleWithPartialData: IUserPreference = {
  id: 25945,
  userId: 'administration',
  remoteOnly: false,
  contractType: 'membre de l’équipe crac',
  preferredRoles: '../fake-data/blob/hipster.txt',
  excludedTechnologies: '../fake-data/blob/hipster.txt',
};

export const sampleWithFullData: IUserPreference = {
  id: 28021,
  userId: 'propre ouin au cas où',
  remoteOnly: false,
  contractType: 'si',
  salaryMin: 27181,
  salaryMax: 13748,
  preferredRoles: '../fake-data/blob/hipster.txt',
  excludedTechnologies: '../fake-data/blob/hipster.txt',
  preferredLocations: '../fake-data/blob/hipster.txt',
};

export const sampleWithNewData: NewUserPreference = {
  userId: 'en decà de',
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
