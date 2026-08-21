import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IRadarHit, NewRadarHit } from '../radar-hit.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IRadarHit for edit and NewRadarHitFormGroupInput for create.
 */
type RadarHitFormGroupInput = IRadarHit | PartialWithRequiredKeyOf<NewRadarHit>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IRadarHit | NewRadarHit> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

type RadarHitFormRawValue = FormValueOf<IRadarHit>;

type NewRadarHitFormRawValue = FormValueOf<NewRadarHit>;

type RadarHitFormDefaults = Pick<NewRadarHit, 'id' | 'seen' | 'dismissed' | 'createdAt'>;

type RadarHitFormGroupContent = {
  id: FormControl<RadarHitFormRawValue['id'] | NewRadarHit['id']>;
  userId: FormControl<RadarHitFormRawValue['userId']>;
  score: FormControl<RadarHitFormRawValue['score']>;
  whyYou: FormControl<RadarHitFormRawValue['whyYou']>;
  seen: FormControl<RadarHitFormRawValue['seen']>;
  dismissed: FormControl<RadarHitFormRawValue['dismissed']>;
  createdAt: FormControl<RadarHitFormRawValue['createdAt']>;
  jobOffer: FormControl<RadarHitFormRawValue['jobOffer']>;
};

export type RadarHitFormGroup = FormGroup<RadarHitFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class RadarHitFormService {
  createRadarHitFormGroup(radarHit?: RadarHitFormGroupInput): RadarHitFormGroup {
    const radarHitRawValue = this.convertRadarHitToRadarHitRawValue({
      ...this.getFormDefaults(),
      ...(radarHit ?? { id: null }),
    });

    return new FormGroup<RadarHitFormGroupContent>({
      id: new FormControl(
        { value: radarHitRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(radarHitRawValue.userId, {
        validators: [Validators.required],
      }),
      score: new FormControl(radarHitRawValue.score),
      whyYou: new FormControl(radarHitRawValue.whyYou),
      seen: new FormControl(radarHitRawValue.seen),
      dismissed: new FormControl(radarHitRawValue.dismissed),
      createdAt: new FormControl(radarHitRawValue.createdAt),
      jobOffer: new FormControl(radarHitRawValue.jobOffer),
    });
  }

  getRadarHit(form: RadarHitFormGroup): IRadarHit | NewRadarHit {
    return this.convertRadarHitRawValueToRadarHit(form.getRawValue());
  }

  resetForm(form: RadarHitFormGroup, radarHit: RadarHitFormGroupInput): void {
    const radarHitRawValue = this.convertRadarHitToRadarHitRawValue({ ...this.getFormDefaults(), ...radarHit });
    form.reset({
      ...radarHitRawValue,
      id: { value: radarHitRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): RadarHitFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      seen: false,
      dismissed: false,
      createdAt: currentTime,
    };
  }

  private convertRadarHitRawValueToRadarHit(rawRadarHit: RadarHitFormRawValue | NewRadarHitFormRawValue): IRadarHit | NewRadarHit {
    return {
      ...rawRadarHit,
      createdAt: dayjs(rawRadarHit.createdAt, DATE_TIME_FORMAT),
    };
  }

  private convertRadarHitToRadarHitRawValue(
    radarHit: IRadarHit | (Partial<NewRadarHit> & RadarHitFormDefaults),
  ): RadarHitFormRawValue | PartialWithRequiredKeyOf<NewRadarHitFormRawValue> {
    return {
      ...radarHit,
      createdAt: radarHit.createdAt ? radarHit.createdAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
