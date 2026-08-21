import { IUser } from './user.model';

export const sampleWithRequiredData: IUser = {
  id: 24814,
  login: 'Ludovic36',
};

export const sampleWithPartialData: IUser = {
  id: 966,
  login: 'Felix_Prevost',
};

export const sampleWithFullData: IUser = {
  id: 5440,
  login: 'Isidore.Cousin20',
};
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
