import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { HttpResponse } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { IOfferTailoredResume } from '../offer-tailored-resume.model';
import { OfferTailoredResumeService } from '../service/offer-tailored-resume.service';

import { OfferTailoredResumeFormService } from './offer-tailored-resume-form.service';
import { OfferTailoredResumeUpdate } from './offer-tailored-resume-update';

describe('OfferTailoredResume Management Update Component', () => {
  let comp: OfferTailoredResumeUpdate;
  let fixture: ComponentFixture<OfferTailoredResumeUpdate>;
  let activatedRoute: ActivatedRoute;
  let offerTailoredResumeFormService: OfferTailoredResumeFormService;
  let offerTailoredResumeService: OfferTailoredResumeService;
  let jobOfferService: JobOfferService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(OfferTailoredResumeUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    offerTailoredResumeFormService = TestBed.inject(OfferTailoredResumeFormService);
    offerTailoredResumeService = TestBed.inject(OfferTailoredResumeService);
    jobOfferService = TestBed.inject(JobOfferService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should call JobOffer query and add missing value', () => {
      const offerTailoredResume: IOfferTailoredResume = { id: 2726 };
      const jobOffer: IJobOffer = { id: 9246 };
      offerTailoredResume.jobOffer = jobOffer;

      const jobOfferCollection: IJobOffer[] = [{ id: 9246 }];
      vitest.spyOn(jobOfferService, 'query').mockReturnValue(of(new HttpResponse({ body: jobOfferCollection })));
      const additionalJobOffers = [jobOffer];
      const expectedCollection: IJobOffer[] = [...additionalJobOffers, ...jobOfferCollection];
      vitest.spyOn(jobOfferService, 'addJobOfferToCollectionIfMissing').mockReturnValue(expectedCollection);

      activatedRoute.data = of({ offerTailoredResume });
      comp.ngOnInit();

      expect(jobOfferService.query).toHaveBeenCalled();
      expect(jobOfferService.addJobOfferToCollectionIfMissing).toHaveBeenCalledWith(
        jobOfferCollection,
        ...additionalJobOffers.map(i => expect.objectContaining(i) as typeof i),
      );
      expect(comp.jobOffersSharedCollection()).toEqual(expectedCollection);
    });

    it('should update editForm', () => {
      const offerTailoredResume: IOfferTailoredResume = { id: 2726 };
      const jobOffer: IJobOffer = { id: 9246 };
      offerTailoredResume.jobOffer = jobOffer;

      activatedRoute.data = of({ offerTailoredResume });
      comp.ngOnInit();

      expect(comp.jobOffersSharedCollection()).toContainEqual(jobOffer);
      expect(comp.offerTailoredResume).toEqual(offerTailoredResume);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IOfferTailoredResume>();
      const offerTailoredResume = { id: 5742 };
      vitest.spyOn(offerTailoredResumeFormService, 'getOfferTailoredResume').mockReturnValue(offerTailoredResume);
      vitest.spyOn(offerTailoredResumeService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ offerTailoredResume });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(offerTailoredResume);
      saveSubject.complete();

      // THEN
      expect(offerTailoredResumeFormService.getOfferTailoredResume).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(offerTailoredResumeService.update).toHaveBeenCalledWith(expect.objectContaining(offerTailoredResume));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IOfferTailoredResume>();
      const offerTailoredResume = { id: 5742 };
      vitest.spyOn(offerTailoredResumeFormService, 'getOfferTailoredResume').mockReturnValue({ id: null });
      vitest.spyOn(offerTailoredResumeService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ offerTailoredResume: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(offerTailoredResume);
      saveSubject.complete();

      // THEN
      expect(offerTailoredResumeFormService.getOfferTailoredResume).toHaveBeenCalled();
      expect(offerTailoredResumeService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IOfferTailoredResume>();
      const offerTailoredResume = { id: 5742 };
      vitest.spyOn(offerTailoredResumeService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ offerTailoredResume });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(offerTailoredResumeService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });

  describe('Compare relationships', () => {
    describe('compareJobOffer', () => {
      it('should forward to jobOfferService', () => {
        const entity = { id: 9246 };
        const entity2 = { id: 5985 };
        vitest.spyOn(jobOfferService, 'compareJobOffer');
        comp.compareJobOffer(entity, entity2);
        expect(jobOfferService.compareJobOffer).toHaveBeenCalledWith(entity, entity2);
      });
    });
  });
});
