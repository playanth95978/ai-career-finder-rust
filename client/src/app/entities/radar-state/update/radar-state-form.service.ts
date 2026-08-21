import { Injectable } from '@angular/core';
import { FormControl, FormGroup, Validators } from '@angular/forms';

import dayjs from 'dayjs/esm';

import { DATE_TIME_FORMAT } from 'app/config/input.constants';
import { IRadarState, NewRadarState } from '../radar-state.model';

/**
 * A partial Type with required key is used as form input.
 */
type PartialWithRequiredKeyOf<T extends { id: unknown }> = Partial<Omit<T, 'id'>> & { id: T['id'] };

/**
 * Type for createFormGroup and resetForm argument.
 * It accepts IRadarState for edit and NewRadarStateFormGroupInput for create.
 */
type RadarStateFormGroupInput = IRadarState | PartialWithRequiredKeyOf<NewRadarState>;

/**
 * Type that converts some properties for forms.
 */
type FormValueOf<T extends IRadarState | NewRadarState> = Omit<T, 'lastOfferAt'> & {
  lastOfferAt?: string | null;
};

type RadarStateFormRawValue = FormValueOf<IRadarState>;

type NewRadarStateFormRawValue = FormValueOf<NewRadarState>;

type RadarStateFormDefaults = Pick<NewRadarState, 'id' | 'lastOfferAt'>;

type RadarStateFormGroupContent = {
  id: FormControl<RadarStateFormRawValue['id'] | NewRadarState['id']>;
  userId: FormControl<RadarStateFormRawValue['userId']>;
  lastOfferAt: FormControl<RadarStateFormRawValue['lastOfferAt']>;
};

export type RadarStateFormGroup = FormGroup<RadarStateFormGroupContent>;

@Injectable({ providedIn: 'root' })
export class RadarStateFormService {
  createRadarStateFormGroup(radarState?: RadarStateFormGroupInput): RadarStateFormGroup {
    const radarStateRawValue = this.convertRadarStateToRadarStateRawValue({
      ...this.getFormDefaults(),
      ...(radarState ?? { id: null }),
    });

    return new FormGroup<RadarStateFormGroupContent>({
      id: new FormControl(
        { value: radarStateRawValue.id, disabled: true },
        {
          nonNullable: true,
          validators: [Validators.required],
        },
      ),
      userId: new FormControl(radarStateRawValue.userId, {
        validators: [Validators.required],
      }),
      lastOfferAt: new FormControl(radarStateRawValue.lastOfferAt),
    });
  }

  getRadarState(form: RadarStateFormGroup): IRadarState | NewRadarState {
    return this.convertRadarStateRawValueToRadarState(form.getRawValue());
  }

  resetForm(form: RadarStateFormGroup, radarState: RadarStateFormGroupInput): void {
    const radarStateRawValue = this.convertRadarStateToRadarStateRawValue({ ...this.getFormDefaults(), ...radarState });
    form.reset({
      ...radarStateRawValue,
      id: { value: radarStateRawValue.id, disabled: true },
    });
  }

  private getFormDefaults(): RadarStateFormDefaults {
    const currentTime = dayjs();

    return {
      id: null,
      lastOfferAt: currentTime,
    };
  }

  private convertRadarStateRawValueToRadarState(
    rawRadarState: RadarStateFormRawValue | NewRadarStateFormRawValue,
  ): IRadarState | NewRadarState {
    return {
      ...rawRadarState,
      lastOfferAt: dayjs(rawRadarState.lastOfferAt, DATE_TIME_FORMAT),
    };
  }

  private convertRadarStateToRadarStateRawValue(
    radarState: IRadarState | (Partial<NewRadarState> & RadarStateFormDefaults),
  ): RadarStateFormRawValue | PartialWithRequiredKeyOf<NewRadarStateFormRawValue> {
    return {
      ...radarState,
      lastOfferAt: radarState.lastOfferAt ? radarState.lastOfferAt.format(DATE_TIME_FORMAT) : undefined,
    };
  }
}
