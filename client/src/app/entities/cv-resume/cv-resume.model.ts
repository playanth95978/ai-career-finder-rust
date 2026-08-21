import dayjs from 'dayjs/esm';

export interface ICvResume {
  id: number;
  userId?: string | null;
  title?: string | null;
  template?: string | null;
  data?: string | null;
  versionNumber?: number | null;
  createdAt?: dayjs.Dayjs | null;
  updatedAt?: dayjs.Dayjs | null;
}

export type NewCvResume = Omit<ICvResume, 'id'> & { id: null };
