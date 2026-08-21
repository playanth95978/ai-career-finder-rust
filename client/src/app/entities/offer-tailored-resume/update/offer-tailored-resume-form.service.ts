import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IOfferTailoredResume, NewOfferTailoredResume } from '../offer-tailored-resume.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IOfferTailoredResume for edit and NewOfferTailoredResumeFormGroupInput for create.
 */
type OfferTailoredResumeFormGroupInput = IOfferTailoredResume | PartialWithRequiredKeyOf<NewOfferTailoredResume>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IOfferTailoredResume | NewOfferTailoredResume> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

type OfferTailoredResumeFormRawValue = FormValueOf<IOfferTailoredResume>;

type NewOfferTailoredResumeFormRawValue = FormValueOf<NewOfferTailoredResume>;

type OfferTailoredResumeFormDefaults = Pick<NewOfferTailoredResume, 'id' | 'createdAt'>;

type OfferTailoredResumeFormGroupContent = {
  id: FormControl<OfferTailoredResumeFormRawValue['id'] | NewOfferTailoredResume['id']>;
  userId: FormControl<OfferTailoredResumeFormRawValue['userId']>;
  data: FormControl<OfferTailoredResumeFormRawValue['data']>;
  title: FormControl<OfferTailoredResumeFormRawValue['title']>;
  createdAt: FormControl<OfferTailoredResumeFormRawValue['createdAt']>;
  jobOffer: FormControl<OfferTailoredResumeFormRawValue['jobOffer']>;
};

export type OfferTailoredResumeFormGroup = FormGroup<OfferTailoredResumeFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class OfferTailoredResumeFormService {
  createOfferTailoredResumeFormGroup(offerTailoredResume?: OfferTailoredResumeFormGroupInput): OfferTailoredResumeFormGroup {
    const offerTailoredResumeRawValue = this.convertOfferTailoredResumeToOfferTailoredResumeRawValue({
      ...this.getFormDefaults(),
      ...(offerTailoredResume ?? { id: null }),
    });

    return new FormGroup<OfferTailoredResumeFormGroupContent>({
      id: new FormControl(
        { value: offerTailoredResumeRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(offerTailoredResumeRawValue.userId, {
        validators: [Validators.required],
      }),
      data: new FormControl(offerTailoredResumeRawValue.data, {
        validators: [Validators.required],
      }),
      title: new FormControl(offerTailoredResumeRawValue.title),
      createdAt: new FormControl(offerTailoredResumeRawValue.createdAt),
      jobOffer: new FormControl(offerTailoredResumeRawValue.jobOffer),
    });
  }

  getOfferTailoredResume(form: OfferTailoredResumeFormGroup): IOfferTailoredResume | NewOfferTailoredResume {
    return this.convertOfferTailoredResumeRawValueToOfferTailoredResume(form.getRawValue());
  }

  resetForm(form: OfferTailoredResumeFormGroup, offerTailoredResume: OfferTailoredResumeFormGroupInput): void {
    const offerTailoredResumeRawValue = this.convertOfferTailoredResumeToOfferTailoredResumeRawValue({
      ...this.getFormDefaults(),
      ...offerTailoredResume,
    });
    form.reset({
      ...offerTailoredResumeRawValue,
      id: { value: offerTailoredResumeRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): OfferTailoredResumeFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      createdAt: currentTime,
    };
  }

  private convertOfferTailoredResumeRawValueToOfferTailoredResume(
    rawOfferTailoredResume: OfferTailoredResumeFormRawValue | NewOfferTailoredResumeFormRawValue,
  ): IOfferTailoredResume | NewOfferTailoredResume {
    return {
      ...rawOfferTailoredResume,
      createdAt: dayjs(rawOfferTailoredResume.createdAt, DATE_TIME_FORMAT),
    };
  }

  private convertOfferTailoredResumeToOfferTailoredResumeRawValue(
    offerTailoredResume: IOfferTailoredResume | (Partial<NewOfferTailoredResume> & OfferTailoredResumeFormDefaults),
  ): OfferTailoredResumeFormRawValue | PartialWithRequiredKeyOf<NewOfferTailoredResumeFormRawValue> {
    return {
      ...offerTailoredResume,
      createdAt: offerTailoredResume.createdAt ? offerTailoredResume.createdAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
