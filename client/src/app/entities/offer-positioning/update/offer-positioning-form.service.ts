import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IOfferPositioning, NewOfferPositioning } from '../offer-positioning.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IOfferPositioning for edit and NewOfferPositioningFormGroupInput for create.
 */
type OfferPositioningFormGroupInput = IOfferPositioning | PartialWithRequiredKeyOf<NewOfferPositioning>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IOfferPositioning | NewOfferPositioning> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

type OfferPositioningFormRawValue = FormValueOf<IOfferPositioning>;

type NewOfferPositioningFormRawValue = FormValueOf<NewOfferPositioning>;

type OfferPositioningFormDefaults = Pick<NewOfferPositioning, 'id' | 'createdAt'>;

type OfferPositioningFormGroupContent = {
  id: FormControl<OfferPositioningFormRawValue['id'] | NewOfferPositioning['id']>;
  userId: FormControl<OfferPositioningFormRawValue['userId']>;
  result: FormControl<OfferPositioningFormRawValue['result']>;
  createdAt: FormControl<OfferPositioningFormRawValue['createdAt']>;
  jobOffer: FormControl<OfferPositioningFormRawValue['jobOffer']>;
};

export type OfferPositioningFormGroup = FormGroup<OfferPositioningFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class OfferPositioningFormService {
  createOfferPositioningFormGroup(offerPositioning?: OfferPositioningFormGroupInput): OfferPositioningFormGroup {
    const offerPositioningRawValue = this.convertOfferPositioningToOfferPositioningRawValue({
      ...this.getFormDefaults(),
      ...(offerPositioning ?? { id: null }),
    });

    return new FormGroup<OfferPositioningFormGroupContent>({
      id: new FormControl(
        { value: offerPositioningRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(offerPositioningRawValue.userId, {
        validators: [Validators.required],
      }),
      result: new FormControl(offerPositioningRawValue.result, {
        validators: [Validators.required],
      }),
      createdAt: new FormControl(offerPositioningRawValue.createdAt),
      jobOffer: new FormControl(offerPositioningRawValue.jobOffer),
    });
  }

  getOfferPositioning(form: OfferPositioningFormGroup): IOfferPositioning | NewOfferPositioning {
    return this.convertOfferPositioningRawValueToOfferPositioning(form.getRawValue());
  }

  resetForm(form: OfferPositioningFormGroup, offerPositioning: OfferPositioningFormGroupInput): void {
    const offerPositioningRawValue = this.convertOfferPositioningToOfferPositioningRawValue({
      ...this.getFormDefaults(),
      ...offerPositioning,
    });
    form.reset({
      ...offerPositioningRawValue,
      id: { value: offerPositioningRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): OfferPositioningFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      createdAt: currentTime,
    };
  }

  private convertOfferPositioningRawValueToOfferPositioning(
    rawOfferPositioning: OfferPositioningFormRawValue | NewOfferPositioningFormRawValue,
  ): IOfferPositioning | NewOfferPositioning {
    return {
      ...rawOfferPositioning,
      createdAt: dayjs(rawOfferPositioning.createdAt, DATE_TIME_FORMAT),
    };
  }

  private convertOfferPositioningToOfferPositioningRawValue(
    offerPositioning: IOfferPositioning | (Partial<NewOfferPositioning> & OfferPositioningFormDefaults),
  ): OfferPositioningFormRawValue | PartialWithRequiredKeyOf<NewOfferPositioningFormRawValue> {
    return {
      ...offerPositioning,
      createdAt: offerPositioning.createdAt ? offerPositioning.createdAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
