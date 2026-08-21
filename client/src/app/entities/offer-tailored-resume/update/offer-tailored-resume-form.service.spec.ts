import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../offer-tailored-resume.test-samples';

import { OfferTailoredResumeFormService } from './offer-tailored-resume-form.service';

describe('OfferTailoredResume Form Service', () => {
  let service: OfferTailoredResumeFormService;

  beforeEach(() => {
    service = TestBed.inject(OfferTailoredResumeFormService);
  });

  describe('Service methods', () => {
    describe('createOfferTailoredResumeFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            data: expect.any(Object),
            title: expect.any(Object),
            createdAt: expect.any(Object),
            jobOffer: expect.any(Object),
          }),
        );
      });

      it('passing IOfferTailoredResume should create a new form with FormGroup', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            data: expect.any(Object),
            title: expect.any(Object),
            createdAt: expect.any(Object),
            jobOffer: expect.any(Object),
          }),
        );
      });
    });

    describe('getOfferTailoredResume', () => {
      it('should return NewOfferTailoredResume for default OfferTailoredResume initial value', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup(sampleWithNewData);

        const offerTailoredResume = service.getOfferTailoredResume(formGroup);

        expect(offerTailoredResume).toMatchObject(sampleWithNewData);
      });

      it('should return NewOfferTailoredResume for empty OfferTailoredResume initial value', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup();

        const offerTailoredResume = service.getOfferTailoredResume(formGroup);

        expect(offerTailoredResume).toMatchObject({});
      });

      it('should return IOfferTailoredResume', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup(sampleWithRequiredData);

        const offerTailoredResume = service.getOfferTailoredResume(formGroup);

        expect(offerTailoredResume).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IOfferTailoredResume should not enable id FormControl', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewOfferTailoredResume should disable id FormControl', () => {
        const formGroup = service.createOfferTailoredResumeFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
