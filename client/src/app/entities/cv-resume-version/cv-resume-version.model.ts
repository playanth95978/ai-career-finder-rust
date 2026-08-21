import dayjs from 'dayjs/esm';

import { ICvResume } from 'app/entities/cv-resume/cv-resume.model';

export interface ICvResumeVersion {
  id: number;
  versionNumber?: number | null;
  title?: string | null;
  template?: string | null;
  data?: string | null;
  createdAt?: dayjs.Dayjs | null;
  resume?: ICvResume | null;
}

export type NewCvResumeVersion = Omit<ICvResumeVersion, 'id'> & { id: null };
