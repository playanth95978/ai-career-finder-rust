import dayjs from 'dayjs/esm';

import { TypeChat } from 'app/entities/enumerations/type-chat.model';

export interface IConversation {
  id: number;
  userId?: string | null;
  title?: string | null;
  summary?: string | null;
  metadata?: string | null;
  typeChat?: keyof typeof TypeChat | null;
  createdAt?: dayjs.Dayjs | null;
  lastMessageAt?: dayjs.Dayjs | null;
}

export type NewConversation = Omit<IConversation, 'id'> & { id: null };
