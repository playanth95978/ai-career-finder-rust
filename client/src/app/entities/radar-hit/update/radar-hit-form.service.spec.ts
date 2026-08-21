import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../radar-hit.test-samples';

import { RadarHitFormService } from './radar-hit-form.service';

describe('RadarHit Form Service', () => {
  let service: RadarHitFormService;

  beforeEach(() => {
    service = TestBed.inject(RadarHitFormService);
  });

  describe('Service methods', () => {
    describe('createRadarHitFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createRadarHitFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            score: expect.any(Object),
            whyYou: expect.any(Object),
            seen: expect.any(Object),
            dismissed: expect.any(Object),
            createdAt: expect.any(Object),
            jobOffer: expect.any(Object),
          }),
        );
      });

      it('passing IRadarHit should create a new form with FormGroup', () => {
        const formGroup = service.createRadarHitFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            score: expect.any(Object),
            whyYou: expect.any(Object),
            seen: expect.any(Object),
            dismissed: expect.any(Object),
            createdAt: expect.any(Object),
            jobOffer: expect.any(Object),
          }),
        );
      });
    });

    describe('getRadarHit', () => {
      it('should return NewRadarHit for default RadarHit initial value', () => {
        const formGroup = service.createRadarHitFormGroup(sampleWithNewData);

        const radarHit = service.getRadarHit(formGroup);

        expect(radarHit).toMatchObject(sampleWithNewData);
      });

      it('should return NewRadarHit for empty RadarHit initial value', () => {
        const formGroup = service.createRadarHitFormGroup();

        const radarHit = service.getRadarHit(formGroup);

        expect(radarHit).toMatchObject({});
      });

      it('should return IRadarHit', () => {
        const formGroup = service.createRadarHitFormGroup(sampleWithRequiredData);

        const radarHit = service.getRadarHit(formGroup);

        expect(radarHit).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IRadarHit should not enable id FormControl', () => {
        const formGroup = service.createRadarHitFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewRadarHit should disable id FormControl', () => {
        const formGroup = service.createRadarHitFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
