import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../radar-state.test-samples';

import { RadarStateFormService } from './radar-state-form.service';

describe('RadarState Form Service', () => {
  let service: RadarStateFormService;

  beforeEach(() => {
    service = TestBed.inject(RadarStateFormService);
  });

  describe('Service methods', () => {
    describe('createRadarStateFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createRadarStateFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            lastOfferAt: expect.any(Object),
          }),
        );
      });

      it('passing IRadarState should create a new form with FormGroup', () => {
        const formGroup = service.createRadarStateFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            lastOfferAt: expect.any(Object),
          }),
        );
      });
    });

    describe('getRadarState', () => {
      it('should return NewRadarState for default RadarState initial value', () => {
        const formGroup = service.createRadarStateFormGroup(sampleWithNewData);

        const radarState = service.getRadarState(formGroup);

        expect(radarState).toMatchObject(sampleWithNewData);
      });

      it('should return NewRadarState for empty RadarState initial value', () => {
        const formGroup = service.createRadarStateFormGroup();

        const radarState = service.getRadarState(formGroup);

        expect(radarState).toMatchObject({});
      });

      it('should return IRadarState', () => {
        const formGroup = service.createRadarStateFormGroup(sampleWithRequiredData);

        const radarState = service.getRadarState(formGroup);

        expect(radarState).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IRadarState should not enable id FormControl', () => {
        const formGroup = service.createRadarStateFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewRadarState should disable id FormControl', () => {
        const formGroup = service.createRadarStateFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
