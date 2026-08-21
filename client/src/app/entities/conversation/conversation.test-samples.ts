import dayjs from 'dayjs/esm';

import { IConversation, NewConversation } from './conversation.model';

export const sampleWithRequiredData: IConversation = {
  id: 7736,
  userId: 'sur',
  createdAt: dayjs('2023-12-11T05:44'),
};

export const sampleWithPartialData: IConversation = {
  id: 18846,
  userId: 'afin de réaliser',
  typeChat: 'CODE',
  createdAt: dayjs('2023-12-11T02:22'),
};

export const sampleWithFullData: IConversation = {
  id: 18032,
  userId: 'beaucoup',
  title: 'démontrer trop peu ronron',
  summary: '../fake-data/blob/hipster.txt',
  metadata: '../fake-data/blob/hipster.txt',
  typeChat: 'CODE',
  createdAt: dayjs('2023-12-10T17:32'),
  lastMessageAt: dayjs('2023-12-10T23:51'),
};

export const sampleWithNewData: NewConversation = {
  userId: 'orange tic-tac pas mal',
  createdAt: dayjs('2023-12-11T11:03'),
  id: null,
};

Object.freeze(sampleWithNewData);
Object.freeze(sampleWithRequiredData);
Object.freeze(sampleWithPartialData);
Object.freeze(sampleWithFullData);
