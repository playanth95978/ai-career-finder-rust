import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IConversation, NewConversation } from '../conversation.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IConversation for edit and NewConversationFormGroupInput for create.
 */
type ConversationFormGroupInput = IConversation | PartialWithRequiredKeyOf<NewConversation>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IConversation | NewConversation> = Omit<T, 'createdAt' | 'lastMessageAt'> & {
  createdAt?: string | null;
  lastMessageAt?: string | null;
};

type ConversationFormRawValue = FormValueOf<IConversation>;

type NewConversationFormRawValue = FormValueOf<NewConversation>;

type ConversationFormDefaults = Pick<NewConversation, 'id' | 'createdAt' | 'lastMessageAt'>;

type ConversationFormGroupContent = {
  id: FormControl<ConversationFormRawValue['id'] | NewConversation['id']>;
  userId: FormControl<ConversationFormRawValue['userId']>;
  title: FormControl<ConversationFormRawValue['title']>;
  summary: FormControl<ConversationFormRawValue['summary']>;
  metadata: FormControl<ConversationFormRawValue['metadata']>;
  typeChat: FormControl<ConversationFormRawValue['typeChat']>;
  createdAt: FormControl<ConversationFormRawValue['createdAt']>;
  lastMessageAt: FormControl<ConversationFormRawValue['lastMessageAt']>;
};

export type ConversationFormGroup = FormGroup<ConversationFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class ConversationFormService {
  createConversationFormGroup(conversation?: ConversationFormGroupInput): ConversationFormGroup {
    const conversationRawValue = this.convertConversationToConversationRawValue({
      ...this.getFormDefaults(),
      ...(conversation ?? { id: null }),
    });

    return new FormGroup<ConversationFormGroupContent>({
      id: new FormControl(
        { value: conversationRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(conversationRawValue.userId, {
        validators: [Validators.required],
      }),
      title: new FormControl(conversationRawValue.title),
      summary: new FormControl(conversationRawValue.summary),
      metadata: new FormControl(conversationRawValue.metadata),
      typeChat: new FormControl(conversationRawValue.typeChat),
      createdAt: new FormControl(conversationRawValue.createdAt, {
        validators: [Validators.required],
      }),
      lastMessageAt: new FormControl(conversationRawValue.lastMessageAt),
    });
  }

  getConversation(form: ConversationFormGroup): IConversation | NewConversation {
    return this.convertConversationRawValueToConversation(form.getRawValue());
  }

  resetForm(form: ConversationFormGroup, conversation: ConversationFormGroupInput): void {
    const conversationRawValue = this.convertConversationToConversationRawValue({ ...this.getFormDefaults(), ...conversation });
    form.reset({
      ...conversationRawValue,
      id: { value: conversationRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): ConversationFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      createdAt: currentTime,
      lastMessageAt: currentTime,
    };
  }

  private convertConversationRawValueToConversation(
    rawConversation: ConversationFormRawValue | NewConversationFormRawValue,
  ): IConversation | NewConversation {
    return {
      ...rawConversation,
      createdAt: dayjs(rawConversation.createdAt, DATE_TIME_FORMAT),
      lastMessageAt: dayjs(rawConversation.lastMessageAt, DATE_TIME_FORMAT),
    };
  }

  private convertConversationToConversationRawValue(
    conversation: IConversation | (Partial<NewConversation> & ConversationFormDefaults),
  ): ConversationFormRawValue | PartialWithRequiredKeyOf<NewConversationFormRawValue> {
    return {
      ...conversation,
      createdAt: conversation.createdAt ? conversation.createdAt.format(DATE_TIME_FORMAT) : undefined,
      lastMessageAt: conversation.lastMessageAt ? conversation.lastMessageAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
